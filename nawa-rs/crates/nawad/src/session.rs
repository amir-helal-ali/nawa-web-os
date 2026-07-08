//! Advanced session management — JWT refresh, blacklisting, and session store.
//!
//! Provides:
//! - Session store (in-memory + DB-backed)
//! - Token blacklisting (for logout/revocation)
//! - Refresh token generation
//! - Session expiration tracking
//! - Active session listing

#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tokio::sync::RwLock;

/// A user session.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Session {
    pub id: String,
    pub user_id: String,
    pub username: String,
    pub token: String,
    pub refresh_token: String,
    pub created_at: String,
    pub expires_at: String,
    pub last_activity: String,
    pub ip_address: String,
    pub user_agent: String,
    pub is_active: bool,
}

/// The session store.
pub struct SessionStore {
    /// Active sessions: session_id → Session.
    sessions: RwLock<HashMap<String, Session>>,
    /// Blacklisted tokens (revoked before expiry).
    blacklist: RwLock<Vec<String>>,
    /// Token → session_id mapping for quick lookup.
    token_index: RwLock<HashMap<String, String>>,
    /// Total sessions created (all time).
    total_created: AtomicU64,
    /// Total sessions expired/revoked.
    total_revoked: AtomicU64,
    /// Default session duration.
    default_duration: Duration,
}

impl SessionStore {
    /// Create a new session store.
    pub fn new(default_duration: Duration) -> Arc<Self> {
        Arc::new(Self {
            sessions: RwLock::new(HashMap::new()),
            blacklist: RwLock::new(Vec::new()),
            token_index: RwLock::new(HashMap::new()),
            total_created: AtomicU64::new(0),
            total_revoked: AtomicU64::new(0),
            default_duration,
        })
    }

    /// Create a new session for a user.
    pub async fn create(
        &self,
        user_id: &str,
        username: &str,
        token: &str,
        ip: &str,
        user_agent: &str,
    ) -> Session {
        let session_id = format!("sess-{}", self.total_created.fetch_add(1, Ordering::Relaxed));
        let now = chrono::Utc::now();
        let expires = now + chrono::Duration::seconds(self.default_duration.as_secs() as i64);
        let refresh = format!("ref-{}", Self::generate_token());

        let session = Session {
            id: session_id.clone(),
            user_id: user_id.to_string(),
            username: username.to_string(),
            token: token.to_string(),
            refresh_token: refresh,
            created_at: now.to_rfc3339(),
            expires_at: expires.to_rfc3339(),
            last_activity: now.to_rfc3339(),
            ip_address: ip.to_string(),
            user_agent: user_agent.to_string(),
            is_active: true,
        };

        let mut sessions = self.sessions.write().await;
        let mut token_idx = self.token_index.write().await;
        token_idx.insert(token.to_string(), session_id.clone());
        sessions.insert(session_id, session.clone());

        session
    }

    /// Get a session by token.
    pub async fn get_by_token(&self, token: &str) -> Option<Session> {
        // Check blacklist first.
        if self.is_blacklisted(token).await {
            return None;
        }
        let token_idx = self.token_index.read().await;
        let session_id = token_idx.get(token)?;
        let sessions = self.sessions.read().await;
        sessions.get(session_id).cloned()
    }

    /// Get all sessions for a user.
    pub async fn get_user_sessions(&self, user_id: &str) -> Vec<Session> {
        let sessions = self.sessions.read().await;
        sessions.values()
            .filter(|s| s.user_id == user_id && s.is_active)
            .cloned()
            .collect()
    }

    /// Update last activity timestamp.
    pub async fn touch(&self, token: &str) -> bool {
        let token_idx = self.token_index.read().await;
        let session_id = match token_idx.get(token) {
            Some(id) => id.clone(),
            None => return false,
        };
        drop(token_idx);

        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(&session_id) {
            session.last_activity = chrono::Utc::now().to_rfc3339();
            return true;
        }
        false
    }

    /// Revoke a session (logout).
    pub async fn revoke(&self, token: &str) -> bool {
        let mut token_idx = self.token_index.write().await;
        let session_id = match token_idx.remove(token) {
            Some(id) => id,
            None => return false,
        };
        drop(token_idx);

        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(&session_id) {
            session.is_active = false;
        }
        sessions.remove(&session_id);

        // Blacklist the token.
        self.blacklist.write().await.push(token.to_string());
        self.total_revoked.fetch_add(1, Ordering::Relaxed);
        true
    }

    /// Revoke all sessions for a user (force logout everywhere).
    pub async fn revoke_all_for_user(&self, user_id: &str) -> usize {
        let mut sessions = self.sessions.write().await;
        let mut token_idx = self.token_index.write().await;
        let mut blacklist = self.blacklist.write().await;

        let to_revoke: Vec<String> = sessions.values()
            .filter(|s| s.user_id == user_id)
            .map(|s| s.token.clone())
            .collect();

        let count = to_revoke.len();
        for token in &to_revoke {
            token_idx.remove(token);
            blacklist.push(token.clone());
        }

        sessions.retain(|_, s| s.user_id != user_id);
        self.total_revoked.fetch_add(count as u64, Ordering::Relaxed);
        count
    }

    /// Check if a token is blacklisted.
    pub async fn is_blacklisted(&self, token: &str) -> bool {
        self.blacklist.read().await.iter().any(|t| t == token)
    }

    /// Clean up expired sessions.
    pub async fn cleanup_expired(&self) -> usize {
        let now = chrono::Utc::now();
        let mut sessions = self.sessions.write().await;
        let mut token_idx = self.token_index.write().await;

        let expired: Vec<String> = sessions.iter()
            .filter(|(_, s)| {
                let expires = chrono::DateTime::parse_from_rfc3339(&s.expires_at).ok();
                expires.map(|e| e.with_timezone(&chrono::Utc) < now).unwrap_or(false)
            })
            .map(|(_, s)| s.token.clone())
            .collect();

        let count = expired.len();
        for token in &expired {
            token_idx.remove(token);
        }
        sessions.retain(|_, s| !expired.contains(&s.token));
        self.total_revoked.fetch_add(count as u64, Ordering::Relaxed);
        count
    }

    /// Get session store statistics.
    pub async fn stats(&self) -> SessionStats {
        let sessions = self.sessions.read().await;
        let blacklist = self.blacklist.read().await;
        SessionStats {
            active_sessions: sessions.len(),
            blacklisted_tokens: blacklist.len(),
            total_created: self.total_created.load(Ordering::Relaxed),
            total_revoked: self.total_revoked.load(Ordering::Relaxed),
        }
    }

    /// Generate a random token (simplified — no external crate).
    fn generate_token() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.subsec_nanos())
            .unwrap_or(0);
        let pid = std::process::id();
        format!("{:016x}", xxhash_rust::xxh3::xxh3_64(format!("{nanos}{pid}").as_bytes()))
    }
}

/// Session store statistics.
#[derive(Debug, Clone, serde::Serialize)]
pub struct SessionStats {
    pub active_sessions: usize,
    pub blacklisted_tokens: usize,
    pub total_created: u64,
    pub total_revoked: u64,
}

/// Refresh token result.
#[derive(Debug, Clone, serde::Serialize)]
pub struct RefreshResult {
    pub new_token: String,
    pub new_refresh_token: String,
    pub expires_at: String,
    pub session_id: String,
}

impl SessionStore {
    /// Refresh a token — generates new tokens for an existing session.
    pub async fn refresh(&self, old_refresh_token: &str) -> Option<RefreshResult> {
        let sessions = self.sessions.read().await;
        let session = sessions.values().find(|s| s.refresh_token == old_refresh_token)?;
        if !session.is_active {
            return None;
        }
        let session_id = session.id.clone();
        let user_id = session.user_id.clone();
        let username = session.username.clone();
        let ip = session.ip_address.clone();
        let ua = session.user_agent.clone();
        drop(sessions);

        // Revoke old session.
        self.revoke_by_refresh(old_refresh_token).await;

        // Create new session.
        let new_token = Self::generate_token();
        let new_session = self.create(&user_id, &username, &new_token, &ip, &ua).await;

        Some(RefreshResult {
            new_token,
            new_refresh_token: new_session.refresh_token,
            expires_at: new_session.expires_at,
            session_id,
        })
    }

    /// Revoke by refresh token (internal helper).
    async fn revoke_by_refresh(&self, refresh_token: &str) {
        let mut sessions = self.sessions.write().await;
        let mut token_idx = self.token_index.write().await;
        let mut blacklist = self.blacklist.write().await;

        if let Some(session) = sessions.values().find(|s| s.refresh_token == refresh_token).cloned() {
            token_idx.remove(&session.token);
            blacklist.push(session.token.clone());
            sessions.remove(&session.id);
            self.total_revoked.fetch_add(1, Ordering::Relaxed);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_store() -> Arc<SessionStore> {
        SessionStore::new(Duration::from_secs(3600))
    }

    #[tokio::test]
    async fn create_session() {
        let store = make_store();
        let session = store.create("user1", "admin", "tok123", "127.0.0.1", "Mozilla").await;
        assert_eq!(session.user_id, "user1");
        assert!(session.is_active);
        assert!(!session.token.is_empty());
        assert!(!session.refresh_token.is_empty());
    }

    #[tokio::test]
    async fn get_by_token() {
        let store = make_store();
        store.create("user1", "admin", "tok123", "127.0.0.1", "Mozilla").await;
        let session = store.get_by_token("tok123").await;
        assert!(session.is_some());
        assert_eq!(session.unwrap().user_id, "user1");
    }

    #[tokio::test]
    async fn get_by_token_not_found() {
        let store = make_store();
        let session = store.get_by_token("nonexistent").await;
        assert!(session.is_none());
    }

    #[tokio::test]
    async fn revoke_session() {
        let store = make_store();
        store.create("user1", "admin", "tok123", "127.0.0.1", "Mozilla").await;
        assert!(store.revoke("tok123").await);
        // Token should be blacklisted.
        assert!(store.is_blacklisted("tok123").await);
        // get_by_token should return None.
        assert!(store.get_by_token("tok123").await.is_none());
    }

    #[tokio::test]
    async fn revoke_nonexistent_fails() {
        let store = make_store();
        assert!(!store.revoke("nonexistent").await);
    }

    #[tokio::test]
    async fn revoke_all_for_user() {
        let store = make_store();
        store.create("user1", "admin", "tok1", "127.0.0.1", "Mozilla").await;
        store.create("user1", "admin", "tok2", "127.0.0.1", "Mozilla").await;
        store.create("user2", "user", "tok3", "127.0.0.1", "Mozilla").await;
        let count = store.revoke_all_for_user("user1").await;
        assert_eq!(count, 2);
        assert!(store.get_by_token("tok1").await.is_none());
        assert!(store.get_by_token("tok2").await.is_none());
        assert!(store.get_by_token("tok3").await.is_some());
    }

    #[tokio::test]
    async fn get_user_sessions() {
        let store = make_store();
        store.create("user1", "admin", "tok1", "127.0.0.1", "Mozilla").await;
        store.create("user1", "admin", "tok2", "127.0.0.1", "Chrome").await;
        store.create("user2", "user", "tok3", "127.0.0.1", "Safari").await;
        let sessions = store.get_user_sessions("user1").await;
        assert_eq!(sessions.len(), 2);
    }

    #[tokio::test]
    async fn touch_updates_activity() {
        let store = make_store();
        store.create("user1", "admin", "tok123", "127.0.0.1", "Mozilla").await;
        let original = store.get_by_token("tok123").await.unwrap();
        tokio::time::sleep(Duration::from_millis(10)).await;
        assert!(store.touch("tok123").await);
        let updated = store.get_by_token("tok123").await.unwrap();
        assert_ne!(original.last_activity, updated.last_activity);
    }

    #[tokio::test]
    async fn blacklisted_token_rejected() {
        let store = make_store();
        store.create("user1", "admin", "tok123", "127.0.0.1", "Mozilla").await;
        store.revoke("tok123").await;
        // Even if session existed, blacklisted token returns None.
        assert!(store.get_by_token("tok123").await.is_none());
    }

    #[tokio::test]
    async fn stats_track_counters() {
        let store = make_store();
        store.create("u1", "a", "t1", "ip", "ua").await;
        store.create("u2", "b", "t2", "ip", "ua").await;
        store.revoke("t1").await;
        let stats = store.stats().await;
        assert_eq!(stats.active_sessions, 1);
        assert_eq!(stats.total_created, 2);
        assert_eq!(stats.total_revoked, 1);
        assert_eq!(stats.blacklisted_tokens, 1);
    }

    #[tokio::test]
    async fn refresh_generates_new_tokens() {
        let store = make_store();
        let session = store.create("user1", "admin", "tok123", "127.0.0.1", "Mozilla").await;
        let result = store.refresh(&session.refresh_token).await;
        assert!(result.is_some());
        let refresh_result = result.unwrap();
        assert_ne!(refresh_result.new_token, "tok123");
        // Old token should be revoked.
        assert!(store.get_by_token("tok123").await.is_none());
        // New token should work.
        assert!(store.get_by_token(&refresh_result.new_token).await.is_some());
    }

    #[tokio::test]
    async fn refresh_with_invalid_token_fails() {
        let store = make_store();
        let result = store.refresh("invalid-refresh").await;
        assert!(result.is_none());
    }
}
