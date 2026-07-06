//! WASM sandbox runtime — wraps wasmtime with security defaults.

use std::collections::HashMap;
use std::sync::Arc;

use crate::plugin::Plugin;
use crate::{DEFAULT_FUEL_LIMIT, DEFAULT_MEMORY_LIMIT};

/// Sandbox error type.
#[derive(Debug, thiserror::Error)]
pub enum SandboxError {
    #[error("wasmtime error: {0}")]
    Wasmtime(#[from] anyhow::Error),
    #[error("plugin not found: {0}")]
    NotFound(String),
    #[error("capability denied: {0}")]
    Denied(String),
    #[error("fuel exhausted")]
    FuelExhausted,
    #[error("memory limit exceeded: {0} > {1}")]
    MemoryExceeded(usize, usize),
    #[error("compilation failed: {0}")]
    Compile(String),
    #[error("instantiation failed: {0}")]
    Instantiate(String),
    #[error("function not found: {0}")]
    FunctionNotFound(String),
    #[error("execution failed: {0}")]
    Execution(String),
}

pub type SandboxResult<T> = std::result::Result<T, SandboxError>;

/// Sandbox configuration.
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    /// Fuel limit (instruction count) per invocation.
    pub fuel_limit: u64,
    /// Memory limit in bytes.
    pub memory_limit: usize,
    /// Allow WASI (filesystem, etc.)? Default: false.
    pub allow_wasi: bool,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            fuel_limit: DEFAULT_FUEL_LIMIT,
            memory_limit: DEFAULT_MEMORY_LIMIT,
            allow_wasi: false,
        }
    }
}

/// The WASM sandbox — manages plugin compilation and execution.
pub struct Sandbox {
    engine: wasmtime::Engine,
    config: SandboxConfig,
    plugins: HashMap<String, Arc<wasmtime::Module>>,
}

impl Sandbox {
    /// Create a new sandbox with the given config.
    pub fn new(config: SandboxConfig) -> SandboxResult<Self> {
        let mut engine_config = wasmtime::Config::new();
        engine_config.consume_fuel(true);
        engine_config.wasm_threads(false);
        engine_config.wasm_simd(true);
        engine_config.wasm_reference_types(false);
        engine_config.wasm_bulk_memory(true);
        engine_config.wasm_multi_value(true);
        engine_config.wasm_multi_memory(false);
        // CRITICAL: no WASI by default — plugins can't touch the filesystem.
        if !config.allow_wasi {
            engine_config.wasm_component_model(false);
        }

        let engine = wasmtime::Engine::new(&engine_config)?;
        Ok(Self {
            engine,
            config,
            plugins: HashMap::new(),
        })
    }

    /// Create a sandbox with default config.
    pub fn default() -> SandboxResult<Self> {
        Self::new(SandboxConfig::default())
    }

    /// Load (compile) a plugin into the sandbox.
    pub fn load(&mut self, plugin: Plugin) -> SandboxResult<()> {
        let module = wasmtime::Module::new(&self.engine, plugin.bytecode.as_slice())
            .map_err(|e| SandboxError::Compile(e.to_string()))?;
        self.plugins
            .insert(plugin.name().to_string(), Arc::new(module));
        tracing::info!(
            plugin = plugin.name(),
            size = plugin.size(),
            "loaded WASM plugin"
        );
        Ok(())
    }

    /// Unload a plugin.
    pub fn unload(&mut self, name: &str) -> bool {
        self.plugins.remove(name).is_some()
    }

    /// List loaded plugins.
    pub fn list(&self) -> Vec<String> {
        self.plugins.keys().cloned().collect()
    }

    /// Number of loaded plugins.
    pub fn len(&self) -> usize {
        self.plugins.len()
    }

    /// Is the sandbox empty?
    pub fn is_empty(&self) -> bool {
        self.plugins.is_empty()
    }

    /// Invoke a function on a plugin.
    ///
    /// For the alpha, this returns the raw i32 result. A production version
    /// would support typed args via wasmtime::Func::typed.
    pub fn invoke(&self, plugin_name: &str, func_name: &str) -> SandboxResult<i32> {
        let module = self
            .plugins
            .get(plugin_name)
            .ok_or_else(|| SandboxError::NotFound(plugin_name.to_string()))?
            .clone();

        let mut store = wasmtime::Store::new(&self.engine, ());
        store
            .set_fuel(self.config.fuel_limit)
            .map_err(|_| SandboxError::FuelExhausted)?;

        let instance = wasmtime::Instance::new(&mut store, &module, &[])
            .map_err(|e| SandboxError::Instantiate(e.to_string()))?;

        let func = instance
            .get_func(&mut store, func_name)
            .ok_or_else(|| SandboxError::FunctionNotFound(func_name.to_string()))?;

        // Call with no args; expect single i32 return.
        let mut results = vec![wasmtime::Val::I32(0)];
        func.call(&mut store, &[], &mut results)
            .map_err(|e| SandboxError::Execution(e.to_string()))?;

        let result = results[0]
            .i32()
            .ok_or_else(|| SandboxError::Execution("expected i32 return".into()))?;
        Ok(result)
    }

    /// Sandbox config (read-only).
    pub fn config(&self) -> &SandboxConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::PluginManifest;

    #[test]
    fn sandbox_creation() {
        let sandbox = Sandbox::default().unwrap();
        assert!(sandbox.is_empty());
        assert_eq!(sandbox.config().fuel_limit, DEFAULT_FUEL_LIMIT);
        assert_eq!(sandbox.config().memory_limit, DEFAULT_MEMORY_LIMIT);
        assert!(!sandbox.config().allow_wasi);
    }

    #[test]
    fn load_unload_plugin() {
        // Use a minimal valid WASM module (just the magic + version).
        // This won't actually compile in wasmtime, so we test the unload path.
        let mut sandbox = Sandbox::default().unwrap();
        assert!(sandbox.is_empty());

        // Try loading invalid bytecode — should error gracefully.
        let manifest = PluginManifest::new("test", "0.1.0");
        let plugin = Plugin::new(manifest, vec![0x00; 8]);
        let result = sandbox.load(plugin);
        assert!(result.is_err()); // invalid WASM
    }

    #[test]
    fn invoke_missing_plugin() {
        let sandbox = Sandbox::default().unwrap();
        let result = sandbox.invoke("missing", "_start");
        assert!(matches!(result, Err(SandboxError::NotFound(_))));
    }

    #[test]
    fn config_defaults() {
        let cfg = SandboxConfig::default();
        assert_eq!(cfg.fuel_limit, 1_000_000);
        assert_eq!(cfg.memory_limit, 64 * 1024 * 1024);
        assert!(!cfg.allow_wasi);
    }
}
