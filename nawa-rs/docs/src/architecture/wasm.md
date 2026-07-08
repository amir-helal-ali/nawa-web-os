# nawa-wasm

WASM sandbox for running user plugins safely.

## Security Model

1. **Capability-based**: plugins receive only declared APIs
2. **Sandboxed**: wasmtime provides memory isolation
3. **Resource-limited**: fuel-based execution limits (1M instructions)
4. **No I/O by default**: no filesystem, no network, no env vars

## Usage

```rust
use nawa_wasm::{Sandbox, Plugin, PluginManifest};

let mut sandbox = Sandbox::default()?;

// Load from bytecode
let manifest = PluginManifest::new("auth-jwt", "0.2.1")
    .with_author("@nawa-official")
    .with_capability("db:read");
let plugin = Plugin::new(manifest, bytecode);
sandbox.load(plugin)?;

// Load from file
let manifest = PluginManifest::new("my-plugin", "1.0.0");
sandbox.load_from_file("plugins/my-plugin.wasm", manifest)?;

// Load all from directory
let count = sandbox.load_from_dir("plugins/")?;

// Invoke a function
let result = sandbox.invoke("my-plugin", "process")?;

// List loaded plugins
let plugins = sandbox.list();

// Unload
sandbox.unload("my-plugin");
```

## Configuration

```rust
use nawa_wasm::{Sandbox, SandboxConfig};

let config = SandboxConfig {
    fuel_limit: 2_000_000,       // 2M instructions
    memory_limit: 128 * 1024 * 1024, // 128 MB
    allow_wasi: false,            // no filesystem/network
};
let sandbox = Sandbox::new(config)?;
```

## Manifest Format

```json
{
  "name": "auth-jwt",
  "version": "0.2.1",
  "author": "@nawa-official",
  "description": "JWT auth handler",
  "capabilities": ["db:read", "http:fetch"]
}
```

## Tests
- 8 unit tests
