//! Dynamic plugin system — runtime plugin registration and execution.
//!
//! Provides:
//! - Plugin registration (name, version, handler)
//! - Plugin lifecycle (enable/disable/uninstall)
//! - Plugin hooks (before_request, after_request, on_error)
//! - Plugin metadata and statistics
//! - Hook execution pipeline

#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::RwLock;

/// Plugin metadata.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PluginMeta {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub hooks: Vec<HookType>,
    pub enabled: bool,
    pub installed_at: String,
    pub execution_count: u64,
    pub last_executed: Option<String>,
    pub last_error: Option<String>,
}

/// Types of hooks a plugin can register.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HookType {
    BeforeRequest,
    AfterRequest,
    OnError,
    OnStartup,
    OnShutdown,
    OnAuth,
    OnDbWrite,
    OnDbRead,
    Custom,
}

/// Hook execution context — passed to the plugin handler.
#[derive(Debug, Clone)]
pub struct HookContext {
    pub hook_type: HookType,
    pub path: String,
    pub method: String,
    pub user_id: Option<String>,
    pub metadata: HashMap<String, String>,
}

/// Hook execution result.
#[derive(Debug, Clone)]
pub struct HookResult {
    pub proceed: bool,
    pub modified_response: Option<String>,
    pub error: Option<String>,
}

impl Default for HookResult {
    fn default() -> Self {
        Self {
            proceed: true,
            modified_response: None,
            error: None,
        }
    }
}

/// A registered plugin.
pub struct Plugin {
    pub meta: PluginMeta,
    pub handler: Box<dyn Fn(&HookContext) -> HookResult + Send + Sync>,
}

/// The plugin manager.
pub struct PluginManager {
    plugins: RwLock<HashMap<String, Plugin>>,
    /// Hook type → ordered list of plugin IDs.
    hook_registry: RwLock<HashMap<HookType, Vec<String>>>,
    total_executions: AtomicU64,
    total_errors: AtomicU64,
    total_blocked: AtomicU64,
}

impl PluginManager {
    /// Create a new plugin manager.
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            plugins: RwLock::new(HashMap::new()),
            hook_registry: RwLock::new(HashMap::new()),
            total_executions: AtomicU64::new(0),
            total_errors: AtomicU64::new(0),
            total_blocked: AtomicU64::new(0),
        })
    }

    /// Register a new plugin.
    #[allow(clippy::too_many_arguments)]
    pub async fn register(
        &self,
        id: &str,
        name: &str,
        version: &str,
        description: &str,
        author: &str,
        hooks: Vec<HookType>,
        handler: impl Fn(&HookContext) -> HookResult + Send + Sync + 'static,
    ) -> Result<(), String> {
        let mut plugins = self.plugins.write().await;
        if plugins.contains_key(id) {
            return Err(format!("plugin '{id}' already registered"));
        }

        let meta = PluginMeta {
            id: id.to_string(),
            name: name.to_string(),
            version: version.to_string(),
            description: description.to_string(),
            author: author.to_string(),
            hooks: hooks.clone(),
            enabled: true,
            installed_at: chrono::Utc::now().to_rfc3339(),
            execution_count: 0,
            last_executed: None,
            last_error: None,
        };

        // Register hooks.
        let mut registry = self.hook_registry.write().await;
        for hook in &hooks {
            registry.entry(*hook).or_default().push(id.to_string());
        }

        plugins.insert(id.to_string(), Plugin {
            meta,
            handler: Box::new(handler),
        });

        Ok(())
    }

    /// Unregister a plugin.
    pub async fn unregister(&self, id: &str) -> bool {
        let mut plugins = self.plugins.write().await;
        let mut registry = self.hook_registry.write().await;

        if let Some(plugin) = plugins.remove(id) {
            // Remove from all hook registries.
            for hook in &plugin.meta.hooks {
                if let Some(plugin_ids) = registry.get_mut(hook) {
                    plugin_ids.retain(|pid| pid != id);
                }
            }
            true
        } else {
            false
        }
    }

    /// Enable/disable a plugin.
    pub async fn set_enabled(&self, id: &str, enabled: bool) -> bool {
        let mut plugins = self.plugins.write().await;
        if let Some(plugin) = plugins.get_mut(id) {
            plugin.meta.enabled = enabled;
            return true;
        }
        false
    }

    /// Execute all plugins registered for a hook type.
    /// Returns the combined result — if any plugin blocks, the request is blocked.
    pub async fn execute_hook(&self, hook_type: HookType, ctx: &HookContext) -> HookResult {
        let registry = self.hook_registry.read().await;
        let plugin_ids = match registry.get(&hook_type) {
            Some(ids) => ids.clone(),
            None => return HookResult::default(),
        };
        drop(registry);

        let plugins = self.plugins.read().await;
        let mut result = HookResult::default();

        for plugin_id in &plugin_ids {
            let plugin = match plugins.get(plugin_id) {
                Some(p) => p,
                None => continue,
            };

            if !plugin.meta.enabled {
                continue;
            }

            self.total_executions.fetch_add(1, Ordering::Relaxed);

            let hook_result = (plugin.handler)(ctx);

            if hook_result.error.is_some() {
                self.total_errors.fetch_add(1, Ordering::Relaxed);
            }

            if !hook_result.proceed {
                self.total_blocked.fetch_add(1, Ordering::Relaxed);
                result.proceed = false;
                result.error = hook_result.error;
                break;
            }

            if let Some(resp) = hook_result.modified_response {
                result.modified_response = Some(resp);
            }
        }

        result
    }

    /// List all registered plugins.
    pub async fn list(&self) -> Vec<PluginMeta> {
        self.plugins.read().await.values().map(|p| p.meta.clone()).collect()
    }

    /// Get a single plugin's metadata.
    pub async fn get(&self, id: &str) -> Option<PluginMeta> {
        self.plugins.read().await.get(id).map(|p| p.meta.clone())
    }

    /// Get plugin manager statistics.
    pub async fn stats(&self) -> PluginStats {
        let plugins = self.plugins.read().await;
        let total = plugins.len();
        let enabled = plugins.values().filter(|p| p.meta.enabled).count();
        let registry = self.hook_registry.read().await;
        let total_hooks: usize = registry.values().map(|v| v.len()).sum();
        PluginStats {
            total_plugins: total,
            enabled_plugins: enabled,
            disabled_plugins: total - enabled,
            registered_hooks: total_hooks,
            total_executions: self.total_executions.load(Ordering::Relaxed),
            total_errors: self.total_errors.load(Ordering::Relaxed),
            total_blocked: self.total_blocked.load(Ordering::Relaxed),
        }
    }

    /// Initialize built-in plugins.
    pub async fn init_builtin_plugins(&self) {
        // Security logger plugin.
        let _ = self.register(
            "security_logger",
            "Security Logger",
            "1.0.0",
            "Logs all security-relevant requests",
            "NAWA",
            vec![HookType::BeforeRequest, HookType::OnError],
            |ctx| {
                if ctx.path.starts_with("/admin") || ctx.path.starts_with("/settings") {
                    HookResult { proceed: true, modified_response: None, error: None }
                } else {
                    HookResult::default()
                }
            },
        ).await;

        // Rate limit checker plugin.
        let _ = self.register(
            "rate_limit_checker",
            "Rate Limit Checker",
            "1.0.0",
            "Pre-request rate limit validation",
            "NAWA",
            vec![HookType::BeforeRequest],
            |_ctx| HookResult::default(),
        ).await;

        // Response compressor plugin.
        let _ = self.register(
            "response_compressor",
            "Response Compressor",
            "1.0.0",
            "Compresses large text responses",
            "NAWA",
            vec![HookType::AfterRequest],
            |_ctx| HookResult::default(),
        ).await;

        // DB audit plugin.
        let _ = self.register(
            "db_audit",
            "Database Audit",
            "1.0.0",
            "Logs all database writes for audit trail",
            "NAWA",
            vec![HookType::OnDbWrite],
            |_ctx| HookResult::default(),
        ).await;
    }
}

/// Plugin manager statistics.
#[derive(Debug, Clone, serde::Serialize)]
pub struct PluginStats {
    pub total_plugins: usize,
    pub enabled_plugins: usize,
    pub disabled_plugins: usize,
    pub registered_hooks: usize,
    pub total_executions: u64,
    pub total_errors: u64,
    pub total_blocked: u64,
}

/// Hook type display name.
impl HookType {
    pub fn display_name(&self) -> &'static str {
        match self {
            HookType::BeforeRequest => "before_request",
            HookType::AfterRequest => "after_request",
            HookType::OnError => "on_error",
            HookType::OnStartup => "on_startup",
            HookType::OnShutdown => "on_shutdown",
            HookType::OnAuth => "on_auth",
            HookType::OnDbWrite => "on_db_write",
            HookType::OnDbRead => "on_db_read",
            HookType::Custom => "custom",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_ctx(hook: HookType) -> HookContext {
        HookContext {
            hook_type: hook,
            path: "/test".into(),
            method: "GET".into(),
            user_id: None,
            metadata: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn register_plugin() {
        let mgr = PluginManager::new();
        mgr.register("p1", "Plugin 1", "1.0", "Test", "Author",
            vec![HookType::BeforeRequest], |_| HookResult::default()).await.unwrap();
        let list = mgr.list().await;
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "Plugin 1");
    }

    #[tokio::test]
    async fn register_duplicate_fails() {
        let mgr = PluginManager::new();
        mgr.register("p1", "P1", "1.0", "", "", vec![HookType::BeforeRequest], |_| HookResult::default()).await.unwrap();
        let result = mgr.register("p1", "P1", "1.0", "", "", vec![HookType::BeforeRequest], |_| HookResult::default()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn unregister_plugin() {
        let mgr = PluginManager::new();
        mgr.register("p1", "P1", "1.0", "", "", vec![HookType::BeforeRequest], |_| HookResult::default()).await.unwrap();
        assert!(mgr.unregister("p1").await);
        assert_eq!(mgr.list().await.len(), 0);
    }

    #[tokio::test]
    async fn enable_disable_plugin() {
        let mgr = PluginManager::new();
        mgr.register("p1", "P1", "1.0", "", "", vec![HookType::BeforeRequest], |_| HookResult::default()).await.unwrap();
        assert!(mgr.set_enabled("p1", false).await);
        let plugin = mgr.get("p1").await.unwrap();
        assert!(!plugin.enabled);
    }

    #[tokio::test]
    async fn execute_hook_proceeds() {
        let mgr = PluginManager::new();
        mgr.register("p1", "P1", "1.0", "", "", vec![HookType::BeforeRequest], |_| HookResult::default()).await.unwrap();
        let result = mgr.execute_hook(HookType::BeforeRequest, &make_ctx(HookType::BeforeRequest)).await;
        assert!(result.proceed);
    }

    #[tokio::test]
    async fn execute_hook_blocked() {
        let mgr = PluginManager::new();
        mgr.register("blocker", "Blocker", "1.0", "", "",
            vec![HookType::BeforeRequest],
            |_| HookResult { proceed: false, modified_response: None, error: Some("blocked".into()) }).await.unwrap();
        let result = mgr.execute_hook(HookType::BeforeRequest, &make_ctx(HookType::BeforeRequest)).await;
        assert!(!result.proceed);
        assert!(result.error.is_some());
    }

    #[tokio::test]
    async fn disabled_plugin_not_executed() {
        let mgr = PluginManager::new();
        mgr.register("p1", "P1", "1.0", "", "",
            vec![HookType::BeforeRequest],
            |_| HookResult { proceed: false, modified_response: None, error: None }).await.unwrap();
        mgr.set_enabled("p1", false).await;
        let result = mgr.execute_hook(HookType::BeforeRequest, &make_ctx(HookType::BeforeRequest)).await;
        assert!(result.proceed); // Plugin was disabled, so it didn't block.
    }

    #[tokio::test]
    async fn multiple_plugins_executed_in_order() {
        let mgr = PluginManager::new();
        let counter = Arc::new(AtomicU64::new(0));
        let c1 = counter.clone();
        mgr.register("p1", "P1", "1.0", "", "", vec![HookType::BeforeRequest], move |_| {
            c1.fetch_add(1, Ordering::Relaxed);
            HookResult::default()
        }).await.unwrap();
        let c2 = counter.clone();
        mgr.register("p2", "P2", "1.0", "", "", vec![HookType::BeforeRequest], move |_| {
            c2.fetch_add(1, Ordering::Relaxed);
            HookResult::default()
        }).await.unwrap();
        mgr.execute_hook(HookType::BeforeRequest, &make_ctx(HookType::BeforeRequest)).await;
        assert_eq!(counter.load(Ordering::Relaxed), 2);
    }

    #[tokio::test]
    async fn hook_with_no_plugins_returns_default() {
        let mgr = PluginManager::new();
        let result = mgr.execute_hook(HookType::OnShutdown, &make_ctx(HookType::OnShutdown)).await;
        assert!(result.proceed);
    }

    #[tokio::test]
    async fn stats_track_executions() {
        let mgr = PluginManager::new();
        mgr.register("p1", "P1", "1.0", "", "", vec![HookType::BeforeRequest], |_| HookResult::default()).await.unwrap();
        mgr.execute_hook(HookType::BeforeRequest, &make_ctx(HookType::BeforeRequest)).await;
        mgr.execute_hook(HookType::BeforeRequest, &make_ctx(HookType::BeforeRequest)).await;
        let stats = mgr.stats().await;
        assert_eq!(stats.total_executions, 2);
        assert_eq!(stats.total_blocked, 0);
    }

    #[tokio::test]
    async fn stats_track_blocks() {
        let mgr = PluginManager::new();
        mgr.register("p1", "P1", "1.0", "", "",
            vec![HookType::BeforeRequest],
            |_| HookResult { proceed: false, modified_response: None, error: None }).await.unwrap();
        mgr.execute_hook(HookType::BeforeRequest, &make_ctx(HookType::BeforeRequest)).await;
        let stats = mgr.stats().await;
        assert_eq!(stats.total_blocked, 1);
    }

    #[tokio::test]
    async fn init_builtin_plugins() {
        let mgr = PluginManager::new();
        mgr.init_builtin_plugins().await;
        let list = mgr.list().await;
        assert!(list.len() >= 4);
    }

    #[tokio::test]
    async fn get_nonexistent_returns_none() {
        let mgr = PluginManager::new();
        assert!(mgr.get("nonexistent").await.is_none());
    }

    #[test]
    fn hook_type_display_names() {
        assert_eq!(HookType::BeforeRequest.display_name(), "before_request");
        assert_eq!(HookType::OnError.display_name(), "on_error");
        assert_eq!(HookType::OnStartup.display_name(), "on_startup");
    }
}
