//! User store — manages users in NAWA-DB.
//!
//! Key format:
//! - `user:{id}` — user record (JSON)
//! - `user:email:{email}` — email → id lookup
//! - `user:count` — total user count
//! - `auth:settings` — project settings (JSON)

use crate::{jwt::JwtCodec, password, rbac::Role, AuthConfig};
use nawa_db::{DbEngine, Value};
use serde::{Deserialize, Serialize};

/// User record stored in NAWA-DB.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub role: String,
    pub verified: bool,
    pub created_at: String,
    pub last_login: Option<String>,
}

/// Auth error type.
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("user not found: {0}")]
    UserNotFound(String),
    #[error("email already registered: {0}")]
    EmailExists(String),
    #[error("invalid credentials")]
    InvalidCredentials,
    #[error("account not verified")]
    NotVerified,
    #[error("verification required — contact admin")]
    VerificationRequired,
    #[error("permission denied: {0}")]
    PermissionDenied(String),
    #[error("db error: {0}")]
    Db(String),
    #[error("jwt error: {0}")]
    Jwt(#[from] crate::JwtError),
    #[error("password error: {0}")]
    Password(#[from] crate::PasswordError),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

impl From<nawa_db::DbError> for AuthError {
    fn from(e: nawa_db::DbError) -> Self {
        AuthError::Db(e.to_string())
    }
}

pub type AuthResult<T> = std::result::Result<T, AuthError>;

/// Authentication store — manages users, sessions, and settings.
pub struct AuthStore {
    db: std::sync::Arc<DbEngine>,
    config: AuthConfig,
    jwt: JwtCodec,
}

/// Project settings (controlled by admin).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSettings {
    pub project_name: String,
    pub registration_open: bool,
    pub verification_required: bool,
    pub max_users: Option<usize>,
    pub jwt_expiry_secs: u64,
}

impl Default for ProjectSettings {
    fn default() -> Self {
        Self {
            project_name: "NAWA Project".into(),
            registration_open: true,
            verification_required: false,
            max_users: None,
            jwt_expiry_secs: 7 * 24 * 60 * 60,
        }
    }
}

/// Login result — contains user + JWT token.
#[derive(Debug, Clone, Serialize)]
pub struct LoginResult {
    pub user: User,
    pub token: String,
    pub expires_in: u64,
}

impl AuthStore {
    /// Create a new auth store backed by NAWA-DB.
    pub fn new(db: std::sync::Arc<DbEngine>, config: AuthConfig) -> Self {
        let jwt = JwtCodec::new(&config.jwt_secret);
        Self { db, config, jwt }
    }

    /// Register a new user.
    ///
    /// - First user ever → role = admin, verified = true (auto-verified)
    /// - Subsequent users → role = user, verified = false (needs admin approval)
    /// - Returns LoginResult with JWT (auto-login)
    pub fn register(&self, username: &str, email: &str, password: &str) -> AuthResult<LoginResult> {
        // Check if email already exists.
        let email_key = format!("user:email:{email}");
        if self.db.get(&email_key).is_some() {
            return Err(AuthError::EmailExists(email.to_string()));
        }

        // Check settings.
        let mut settings = self.get_settings()?;
        if !settings.registration_open {
            return Err(AuthError::PermissionDenied("registration is closed".into()));
        }
        if let Some(max) = settings.max_users {
            if self.user_count() >= max {
                return Err(AuthError::PermissionDenied("max users reached".into()));
            }
        }

        // Determine role: first user = admin, rest = user.
        let user_count = self.user_count();
        let (role, verified) = if user_count == 0 {
            (Role::Admin.as_str().to_string(), true) // first user = admin, auto-verified
        } else {
            (Role::User.as_str().to_string(), !settings.verification_required)
        };

        // Generate user ID.
        let id = format!("u{}", user_count + 1);
        let password_hash = password::hash_password(password);
        let now = chrono::Utc::now().to_rfc3339();

        let user = User {
            id: id.clone(),
            username: username.to_string(),
            email: email.to_string(),
            password_hash,
            role,
            verified,
            created_at: now.clone(),
            last_login: Some(now),
        };

        // Store user record.
        let user_json = serde_json::to_string(&user)?;
        self.db.put(format!("user:{id}"), Value::from_json_str(&user_json)?)?;
        // Store email lookup.
        self.db.put(&email_key, Value::from_str(&id))?;
        // Increment count.
        self.db.put("user:count", Value::from_i64((user_count + 1) as i64))?;

        // Auto-enable verification_required after first user (if not already).
        if user_count == 0 && !settings.verification_required {
            settings.verification_required = true;
            self.save_settings(&settings)?;
        }

        // Generate JWT token (auto-login).
        let jwt_expiry = settings.jwt_expiry_secs;
        let token = self.jwt.create_token(&id, &user.role, jwt_expiry)?;

        Ok(LoginResult {
            user,
            token,
            expires_in: jwt_expiry,
        })
    }

    /// Login with email + password.
    pub fn login(&self, email: &str, password: &str) -> AuthResult<LoginResult> {
        // Look up user by email.
        let email_key = format!("user:email:{email}");
        let user_id = self
            .db
            .get(&email_key)
            .ok_or(AuthError::InvalidCredentials)?;
        let user_id = user_id.display();

        // Load user record.
        let user_value = self
            .db
            .get(format!("user:{user_id}"))
            .ok_or(AuthError::UserNotFound(user_id.clone()))?;

        let mut user: User = serde_json::from_str(&user_value.display())?;

        // Verify password.
        if !password::verify_password(password, &user.password_hash)? {
            return Err(AuthError::InvalidCredentials);
        }

        // Check verification.
        let settings = self.get_settings()?;
        if settings.verification_required && !user.verified {
            return Err(AuthError::NotVerified);
        }

        // Update last login.
        user.last_login = Some(chrono::Utc::now().to_rfc3339());
        let user_json = serde_json::to_string(&user)?;
        self.db.put(format!("user:{}", user.id), Value::from_json_str(&user_json)?)?;

        // Generate JWT.
        let token = self.jwt.create_token(&user.id, &user.role, settings.jwt_expiry_secs)?;

        Ok(LoginResult {
            user,
            token,
            expires_in: settings.jwt_expiry_secs,
        })
    }

    /// Verify a JWT token and return the claims.
    pub fn verify_token(&self, token: &str) -> AuthResult<crate::JwtClaims> {
        Ok(self.jwt.decode(token)?)
    }

    /// Get a user by ID.
    pub fn get_user(&self, id: &str) -> AuthResult<User> {
        let value = self
            .db
            .get(format!("user:{id}"))
            .ok_or(AuthError::UserNotFound(id.to_string()))?;
        let user: User = serde_json::from_str(&value.display())?;
        Ok(user)
    }

    /// List all users (admin only).
    pub fn list_users(&self) -> AuthResult<Vec<User>> {
        let entries = self.db.scan_prefix("user:u", 10_000);
        let mut users: Vec<User> = entries
            .iter()
            .filter_map(|(_, v)| {
                serde_json::from_str::<User>(&v.display()).ok()
            })
            .collect();
        users.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        Ok(users)
    }

    /// Change a user's role (admin only).
    pub fn change_role(&self, admin_id: &str, target_user_id: &str, new_role: &str) -> AuthResult<User> {
        // Verify admin.
        let admin = self.get_user(admin_id)?;
        if !Role::from_str(&admin.role).map(|r| r.is_admin()).unwrap_or(false) {
            return Err(AuthError::PermissionDenied("admin role required".into()));
        }

        // Load target user.
        let mut user = self.get_user(target_user_id)?;
        user.role = new_role.to_string();

        let user_json = serde_json::to_string(&user)?;
        self.db.put(format!("user:{}", user.id), Value::from_json_str(&user_json)?)?;

        Ok(user)
    }

    /// Verify a user (admin only).
    pub fn verify_user(&self, admin_id: &str, target_user_id: &str) -> AuthResult<User> {
        let admin = self.get_user(admin_id)?;
        if !Role::from_str(&admin.role).map(|r| r.is_admin()).unwrap_or(false) {
            return Err(AuthError::PermissionDenied("admin role required".into()));
        }

        let mut user = self.get_user(target_user_id)?;
        user.verified = true;
        let user_json = serde_json::to_string(&user)?;
        self.db.put(format!("user:{}", user.id), Value::from_json_str(&user_json)?)?;
        Ok(user)
    }

    /// Delete a user (admin only).
    pub fn delete_user(&self, admin_id: &str, target_user_id: &str) -> AuthResult<()> {
        let admin = self.get_user(admin_id)?;
        if !Role::from_str(&admin.role).map(|r| r.is_admin()).unwrap_or(false) {
            return Err(AuthError::PermissionDenied("admin role required".into()));
        }
        if admin.id == target_user_id {
            return Err(AuthError::PermissionDenied("cannot delete yourself".into()));
        }

        let user = self.get_user(target_user_id)?;
        self.db.delete(format!("user:{}", user.id))?;
        self.db.delete(format!("user:email:{}", user.email))?;
        Ok(())
    }

    /// Get project settings.
    pub fn get_settings(&self) -> AuthResult<ProjectSettings> {
        match self.db.get("auth:settings") {
            Some(v) => Ok(serde_json::from_str(&v.display())?),
            None => Ok(ProjectSettings::default()),
        }
    }

    /// Save project settings (admin only).
    pub fn save_settings(&self, settings: &ProjectSettings) -> AuthResult<()> {
        let json = serde_json::to_string(settings)?;
        self.db.put("auth:settings", Value::from_json_str(&json)?)?;
        Ok(())
    }

    /// Update settings (admin only).
    pub fn update_settings(&self, admin_id: &str, settings: &ProjectSettings) -> AuthResult<ProjectSettings> {
        let admin = self.get_user(admin_id)?;
        if !Role::from_str(&admin.role).map(|r| r.is_admin()).unwrap_or(false) {
            return Err(AuthError::PermissionDenied("admin role required".into()));
        }
        self.save_settings(settings)?;
        Ok(settings.clone())
    }

    /// Number of registered users.
    pub fn user_count(&self) -> usize {
        self.db
            .get("user:count")
            .map(|v| v.display().parse::<usize>().unwrap_or(0))
            .unwrap_or(0)
    }

    /// Get the auth config.
    pub fn config(&self) -> &AuthConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_store() -> AuthStore {
        let db = std::sync::Arc::new(DbEngine::open_in_memory());
        AuthStore::new(db, AuthConfig::with_secret("test-secret"))
    }

    #[test]
    fn first_user_is_admin() {
        let store = make_store();
        let result = store.register("admin", "admin@test.com", "pass123").unwrap();
        assert_eq!(result.user.role, "admin");
        assert!(result.user.verified); // auto-verified
        assert!(!result.token.is_empty()); // auto-login
    }

    #[test]
    fn second_user_is_regular() {
        let store = make_store();
        store.register("admin", "admin@test.com", "pass123").unwrap();
        let result = store.register("user1", "user1@test.com", "pass456").unwrap();
        assert_eq!(result.user.role, "user");
    }

    #[test]
    fn duplicate_email_rejected() {
        let store = make_store();
        store.register("admin", "dup@test.com", "pass").unwrap();
        let result = store.register("second", "dup@test.com", "pass");
        assert!(matches!(result, Err(AuthError::EmailExists(_))));
    }

    #[test]
    fn login_works() {
        let store = make_store();
        store.register("admin", "admin@test.com", "mypass").unwrap();
        let result = store.login("admin@test.com", "mypass").unwrap();
        assert_eq!(result.user.username, "admin");
        assert!(!result.token.is_empty());
    }

    #[test]
    fn login_wrong_password_fails() {
        let store = make_store();
        store.register("admin", "admin@test.com", "correct").unwrap();
        assert!(matches!(store.login("admin@test.com", "wrong"), Err(AuthError::InvalidCredentials)));
    }

    #[test]
    fn token_verification() {
        let store = make_store();
        let result = store.register("admin", "admin@test.com", "pass").unwrap();
        let claims = store.verify_token(&result.token).unwrap();
        assert_eq!(claims.sub, "u1");
        assert_eq!(claims.role, "admin");
    }

    #[test]
    fn list_users() {
        let store = make_store();
        store.register("admin", "a@test.com", "p").unwrap();
        store.register("u1", "u1@test.com", "p").unwrap();
        store.register("u2", "u2@test.com", "p").unwrap();
        let users = store.list_users().unwrap();
        assert_eq!(users.len(), 3);
    }

    #[test]
    fn admin_can_change_role() {
        let store = make_store();
        store.register("admin", "a@test.com", "p").unwrap();
        store.register("u1", "u1@test.com", "p").unwrap();
        let updated = store.change_role("u1", "u2", "admin").unwrap();
        assert_eq!(updated.role, "admin");
    }

    #[test]
    fn non_admin_cannot_change_role() {
        let store = make_store();
        store.register("admin", "a@test.com", "p").unwrap();
        store.register("u1", "u1@test.com", "p").unwrap();
        let result = store.change_role("u2", "u1", "admin");
        assert!(matches!(result, Err(AuthError::PermissionDenied(_))));
    }

    #[test]
    fn cannot_delete_self() {
        let store = make_store();
        store.register("admin", "a@test.com", "p").unwrap();
        let result = store.delete_user("u1", "u1");
        assert!(matches!(result, Err(AuthError::PermissionDenied(_))));
    }

    #[test]
    fn settings_default() {
        let store = make_store();
        let settings = store.get_settings().unwrap();
        assert!(settings.registration_open);
        assert!(!settings.verification_required); // until first user
    }

    #[test]
    fn verification_auto_enabled_after_first_user() {
        let store = make_store();
        store.register("admin", "a@test.com", "p").unwrap();
        let settings = store.get_settings().unwrap();
        assert!(settings.verification_required); // auto-enabled
    }

    #[test]
    fn user_count_tracks_registrations() {
        let store = make_store();
        assert_eq!(store.user_count(), 0);
        store.register("a", "a@t.com", "p").unwrap();
        assert_eq!(store.user_count(), 1);
        store.register("b", "b@t.com", "p").unwrap();
        assert_eq!(store.user_count(), 2);
    }
}
