//! # NAWA Auth
//!
//! Professional authentication system with:
//! - JWT tokens (HS256)
//! - Password hashing (SHA-256 + salt)
//! - Role-based access control (admin / user)
//! - First user = admin, subsequent = user
//! - Auto-login after registration
//! - Session management via NAWA-DB

pub mod jwt;
pub mod password;
pub mod rbac;
pub mod store;

pub use jwt::{JwtCodec, JwtClaims, JwtError};
pub use password::{hash_password, verify_password, PasswordError};
pub use rbac::{Permission, Role};
pub use store::{AuthStore, User, AuthError, AuthResult};

/// Auth configuration.
#[derive(Debug, Clone)]
pub struct AuthConfig {
    /// JWT secret key (HMAC-SHA256).
    pub jwt_secret: String,
    /// Token expiry in seconds (default: 7 days).
    pub token_expiry_secs: u64,
    /// Whether verification is mandatory (auto-enabled after first user).
    pub verification_required: bool,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            jwt_secret: "nawa-default-secret-change-me".to_string(),
            token_expiry_secs: 7 * 24 * 60 * 60, // 7 days
            verification_required: false,
        }
    }
}

impl AuthConfig {
    /// Create a new auth config with a custom JWT secret.
    pub fn with_secret(secret: impl Into<String>) -> Self {
        Self {
            jwt_secret: secret.into(),
            ..Default::default()
        }
    }

    /// Set token expiry.
    pub fn with_expiry(mut self, secs: u64) -> Self {
        self.token_expiry_secs = secs;
        self
    }
}
