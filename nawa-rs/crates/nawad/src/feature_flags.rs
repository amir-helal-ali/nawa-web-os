//! Feature flags — dynamic feature toggling without redeployment.
//!
//! Provides:
//! - Runtime feature flag management
//! - Per-user / per-role flag overrides
//! - Percentage-based rollout
//! - Flag persistence (in-memory + DB-backed)
//! - Flag categories and metadata

#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::RwLock;

/// A feature flag.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FeatureFlag {
    pub key: String,
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub category: FlagCategory,
    pub rollout_percentage: u8,
    pub user_overrides: HashMap<String, bool>,
    pub role_overrides: HashMap<String, bool>,
    pub created_at: String,
    pub updated_at: String,
}

/// Flag category.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FlagCategory {
    Experimental,
    Beta,
    Stable,
    Deprecated,
    Security,
    Performance,
}

/// The feature flag manager.
pub struct FeatureFlags {
    flags: RwLock<HashMap<String, FeatureFlag>>,
    total_evaluations: AtomicU64,
    total_enabled: AtomicU64,
    total_disabled: AtomicU64,
}

impl FeatureFlags {
    /// Create a new feature flag manager.
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            flags: RwLock::new(HashMap::new()),
            total_evaluations: AtomicU64::new(0),
            total_enabled: AtomicU64::new(0),
            total_disabled: AtomicU64::new(0),
        })
    }

    /// Register a new feature flag.
    pub async fn register(&self, key: &str, name: &str, description: &str, category: FlagCategory, enabled: bool) -> bool {
        let mut flags = self.flags.write().await;
        if flags.contains_key(key) {
            return false;
        }
        let now = chrono::Utc::now().to_rfc3339();
        flags.insert(key.to_string(), FeatureFlag {
            key: key.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            enabled,
            category,
            rollout_percentage: if enabled { 100 } else { 0 },
            user_overrides: HashMap::new(),
            role_overrides: HashMap::new(),
            created_at: now.clone(),
            updated_at: now,
        });
        true
    }

    /// Check if a feature is enabled (global check, no user context).
    pub async fn is_enabled(&self, key: &str) -> bool {
        self.total_evaluations.fetch_add(1, Ordering::Relaxed);
        let flags = self.flags.read().await;
        let enabled = flags.get(key).map(|f| f.enabled).unwrap_or(false);
        if enabled {
            self.total_enabled.fetch_add(1, Ordering::Relaxed);
        } else {
            self.total_disabled.fetch_add(1, Ordering::Relaxed);
        }
        enabled
    }

    /// Check if a feature is enabled for a specific user.
    /// Priority: user override > role override > rollout percentage > global flag.
    pub async fn is_enabled_for_user(&self, key: &str, user_id: &str, role: &str) -> bool {
        self.total_evaluations.fetch_add(1, Ordering::Relaxed);
        let flags = self.flags.read().await;
        let flag = match flags.get(key) {
            Some(f) => f,
            None => {
                self.total_disabled.fetch_add(1, Ordering::Relaxed);
                return false;
            }
        };

        // 1. User override takes highest priority.
        if let Some(&override_val) = flag.user_overrides.get(user_id) {
            if override_val {
                self.total_enabled.fetch_add(1, Ordering::Relaxed);
            } else {
                self.total_disabled.fetch_add(1, Ordering::Relaxed);
            }
            return override_val;
        }

        // 2. Role override.
        if let Some(&override_val) = flag.role_overrides.get(role) {
            if override_val {
                self.total_enabled.fetch_add(1, Ordering::Relaxed);
            } else {
                self.total_disabled.fetch_add(1, Ordering::Relaxed);
            }
            return override_val;
        }

        // 3. Rollout percentage (deterministic hash based on user_id).
        if flag.rollout_percentage < 100 {
            let hash = xxhash_rust::xxh3::xxh3_64(user_id.as_bytes());
            let bucket = (hash % 100) as u8;
            let enabled = bucket < flag.rollout_percentage;
            if enabled {
                self.total_enabled.fetch_add(1, Ordering::Relaxed);
            } else {
                self.total_disabled.fetch_add(1, Ordering::Relaxed);
            }
            return enabled;
        }

        // 4. Global flag.
        if flag.enabled {
            self.total_enabled.fetch_add(1, Ordering::Relaxed);
        } else {
            self.total_disabled.fetch_add(1, Ordering::Relaxed);
        }
        flag.enabled
    }

    /// Enable a feature globally.
    pub async fn enable(&self, key: &str) -> bool {
        let mut flags = self.flags.write().await;
        if let Some(flag) = flags.get_mut(key) {
            flag.enabled = true;
            flag.rollout_percentage = 100;
            flag.updated_at = chrono::Utc::now().to_rfc3339();
            return true;
        }
        false
    }

    /// Disable a feature globally.
    pub async fn disable(&self, key: &str) -> bool {
        let mut flags = self.flags.write().await;
        if let Some(flag) = flags.get_mut(key) {
            flag.enabled = false;
            flag.rollout_percentage = 0;
            flag.updated_at = chrono::Utc::now().to_rfc3339();
            return true;
        }
        false
    }

    /// Set rollout percentage for gradual deployment.
    pub async fn set_rollout(&self, key: &str, percentage: u8) -> bool {
        let mut flags = self.flags.write().await;
        if let Some(flag) = flags.get_mut(key) {
            flag.rollout_percentage = percentage.min(100);
            flag.enabled = percentage > 0;
            flag.updated_at = chrono::Utc::now().to_rfc3339();
            return true;
        }
        false
    }

    /// Set a per-user override.
    pub async fn set_user_override(&self, key: &str, user_id: &str, enabled: bool) -> bool {
        let mut flags = self.flags.write().await;
        if let Some(flag) = flags.get_mut(key) {
            flag.user_overrides.insert(user_id.to_string(), enabled);
            flag.updated_at = chrono::Utc::now().to_rfc3339();
            return true;
        }
        false
    }

    /// Set a per-role override.
    pub async fn set_role_override(&self, key: &str, role: &str, enabled: bool) -> bool {
        let mut flags = self.flags.write().await;
        if let Some(flag) = flags.get_mut(key) {
            flag.role_overrides.insert(role.to_string(), enabled);
            flag.updated_at = chrono::Utc::now().to_rfc3339();
            return true;
        }
        false
    }

    /// Get all feature flags.
    pub async fn list(&self) -> Vec<FeatureFlag> {
        self.flags.read().await.values().cloned().collect()
    }

    /// Get a single flag.
    pub async fn get(&self, key: &str) -> Option<FeatureFlag> {
        self.flags.read().await.get(key).cloned()
    }

    /// Get feature flag statistics.
    pub async fn stats(&self) -> FlagStats {
        let flags = self.flags.read().await;
        let total = flags.len();
        let enabled = flags.values().filter(|f| f.enabled).count();
        FlagStats {
            total_flags: total,
            enabled_flags: enabled,
            disabled_flags: total - enabled,
            total_evaluations: self.total_evaluations.load(Ordering::Relaxed),
            total_enabled_evals: self.total_enabled.load(Ordering::Relaxed),
            total_disabled_evals: self.total_disabled.load(Ordering::Relaxed),
        }
    }

    /// Initialize default flags.
    pub async fn init_defaults(&self) {
        self.register("quantum_engine", "Quantum Computing Engine", "Quantum-inspired algorithms", FlagCategory::Stable, true).await;
        self.register("aion_seo", "AION SEO Engine", "Knowledge Graph + multi-format SEO", FlagCategory::Stable, true).await;
        self.register("wasm_ssr", "WASM SSR", "Server-side rendering via WebAssembly", FlagCategory::Beta, true).await;
        self.register("sveltekit", "SvelteKit Integration", "Embedded SvelteKit app support", FlagCategory::Beta, true).await;
        self.register("http3", "HTTP/3 + QUIC", "Next-gen HTTP protocol support", FlagCategory::Experimental, false).await;
        self.register("quantum_tunneling", "Quantum Tunneling Optimizer", "Escape local optima in AION", FlagCategory::Experimental, true).await;
        self.register("pubsub_channels", "WebSocket Pub/Sub", "Topic-based message routing", FlagCategory::Stable, true).await;
        self.register("session_refresh", "Session Token Refresh", "JWT refresh token rotation", FlagCategory::Stable, true).await;
        self.register("openapi_docs", "OpenAPI/Swagger Docs", "Auto-generated API documentation", FlagCategory::Stable, true).await;
        self.register("audit_logging", "Audit Logging", "Security event logging", FlagCategory::Security, true).await;
    }
}

/// Feature flag statistics.
#[derive(Debug, Clone, serde::Serialize)]
pub struct FlagStats {
    pub total_flags: usize,
    pub enabled_flags: usize,
    pub disabled_flags: usize,
    pub total_evaluations: u64,
    pub total_enabled_evals: u64,
    pub total_disabled_evals: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_flags() -> Arc<FeatureFlags> {
        FeatureFlags::new()
    }

    #[tokio::test]
    async fn register_and_check() {
        let flags = make_flags();
        flags.register("test", "Test", "Test flag", FlagCategory::Beta, true).await;
        assert!(flags.is_enabled("test").await);
    }

    #[tokio::test]
    async fn register_duplicate_returns_false() {
        let flags = make_flags();
        flags.register("test", "Test", "", FlagCategory::Beta, true).await;
        assert!(!flags.register("test", "Test", "", FlagCategory::Beta, true).await);
    }

    #[tokio::test]
    async fn is_enabled_nonexistent_returns_false() {
        let flags = make_flags();
        assert!(!flags.is_enabled("nonexistent").await);
    }

    #[tokio::test]
    async fn enable_disable() {
        let flags = make_flags();
        flags.register("test", "Test", "", FlagCategory::Stable, false).await;
        assert!(!flags.is_enabled("test").await);
        flags.enable("test").await;
        assert!(flags.is_enabled("test").await);
        flags.disable("test").await;
        assert!(!flags.is_enabled("test").await);
    }

    #[tokio::test]
    async fn user_override_takes_priority() {
        let flags = make_flags();
        flags.register("test", "Test", "", FlagCategory::Stable, false).await;
        flags.set_user_override("test", "user1", true).await;
        assert!(flags.is_enabled_for_user("test", "user1", "user").await);
    }

    #[tokio::test]
    async fn role_override_works() {
        let flags = make_flags();
        flags.register("test", "Test", "", FlagCategory::Stable, false).await;
        flags.set_role_override("test", "admin", true).await;
        assert!(flags.is_enabled_for_user("test", "user1", "admin").await);
        assert!(!flags.is_enabled_for_user("test", "user2", "user").await);
    }

    #[tokio::test]
    async fn rollout_percentage_deterministic() {
        let flags = make_flags();
        flags.register("test", "Test", "", FlagCategory::Beta, false).await;
        flags.set_rollout("test", 50).await;
        // Same user should get same result every time (deterministic hash).
        let result1 = flags.is_enabled_for_user("test", "user1", "user").await;
        let result2 = flags.is_enabled_for_user("test", "user1", "user").await;
        assert_eq!(result1, result2);
    }

    #[tokio::test]
    async fn rollout_100_enables_all() {
        let flags = make_flags();
        flags.register("test", "Test", "", FlagCategory::Beta, false).await;
        flags.set_rollout("test", 100).await;
        assert!(flags.is_enabled_for_user("test", "user1", "user").await);
        assert!(flags.is_enabled_for_user("test", "user2", "user").await);
    }

    #[tokio::test]
    async fn rollout_0_disables_all() {
        let flags = make_flags();
        flags.register("test", "Test", "", FlagCategory::Beta, true).await;
        flags.set_rollout("test", 0).await;
        assert!(!flags.is_enabled_for_user("test", "user1", "user").await);
    }

    #[tokio::test]
    async fn list_returns_all_flags() {
        let flags = make_flags();
        flags.register("f1", "Flag 1", "", FlagCategory::Stable, true).await;
        flags.register("f2", "Flag 2", "", FlagCategory::Beta, false).await;
        let list = flags.list().await;
        assert_eq!(list.len(), 2);
    }

    #[tokio::test]
    async fn get_returns_flag() {
        let flags = make_flags();
        flags.register("test", "Test Flag", "Description", FlagCategory::Stable, true).await;
        let flag = flags.get("test").await.unwrap();
        assert_eq!(flag.name, "Test Flag");
        assert_eq!(flag.description, "Description");
    }

    #[tokio::test]
    async fn stats_track_evaluations() {
        let flags = make_flags();
        flags.register("on", "On", "", FlagCategory::Stable, true).await;
        flags.register("off", "Off", "", FlagCategory::Stable, false).await;
        flags.is_enabled("on").await;
        flags.is_enabled("off").await;
        let stats = flags.stats().await;
        assert_eq!(stats.total_flags, 2);
        assert_eq!(stats.enabled_flags, 1);
        assert_eq!(stats.total_evaluations, 2);
        assert_eq!(stats.total_enabled_evals, 1);
        assert_eq!(stats.total_disabled_evals, 1);
    }

    #[tokio::test]
    async fn init_defaults_creates_flags() {
        let flags = make_flags();
        flags.init_defaults().await;
        let list = flags.list().await;
        assert!(list.len() >= 8);
        assert!(flags.is_enabled("quantum_engine").await);
        assert!(flags.is_enabled("aion_seo").await);
        assert!(!flags.is_enabled("http3").await); // Experimental, disabled by default.
    }
}
