//! # NAWA WASM Sandbox
//!
//! Runs user-provided code (plugins, handlers) in a secure WebAssembly sandbox.
//! No filesystem access, no network access, no environment variables.
//! Only explicitly-granted capabilities are available.
//!
//! ## Security Model
//!
//! 1. **Capability-based**: plugins receive only the APIs they declare.
//! 2. **Sandboxed**: wasmtime provides memory isolation.
//! 3. **Resource-limited**: fuel-based execution limits.
//! 4. **No I/O**: no filesystem, no network, no env vars by default.

pub mod plugin;
pub mod runtime;

pub use plugin::{Plugin, PluginManifest};
pub use runtime::{Sandbox, SandboxConfig, SandboxError, SandboxResult};

/// Maximum fuel (instruction count) for a single plugin invocation.
pub const DEFAULT_FUEL_LIMIT: u64 = 1_000_000;

/// Maximum memory (bytes) a plugin can allocate.
pub const DEFAULT_MEMORY_LIMIT: usize = 64 * 1024 * 1024; // 64 MB
