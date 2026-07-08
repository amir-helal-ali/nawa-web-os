# WASM Plugins

## Loading Plugins

### From a single file

```bash
# Place .wasm file in your plugins directory
mkdir -p plugins
cp my-plugin.wasm plugins/
```

### Via API

```bash
# List loaded plugins
curl http://localhost:8080/plugins
# → {"count":0,"plugins":[]}

# Invoke a plugin function
curl -X POST http://localhost:8080/plugins/my-plugin/invoke -d 'process'
# → {"plugin":"my-plugin","function":"process","result":0,"status":"ok"}
```

## Plugin Manifest

Each plugin should have a manifest (JSON):

```json
{
  "name": "auth-jwt",
  "version": "0.2.1",
  "author": "@nawa-official",
  "description": "JWT authentication handler",
  "capabilities": ["db:read", "http:fetch"]
}
```

## Security Model

- **No filesystem access** — WASI disabled by default
- **No network access** — no raw sockets
- **Fuel-limited** — 1M instructions per invocation
- **Memory-limited** — 64 MB max per plugin
- **Capability-based** — only declared APIs available

## Writing a Plugin

Plugins are written in any language that compiles to WASM (Rust, C, AssemblyScript, etc.):

```rust
// plugin.rs (compiled with: rustc --target wasm32-unknown-unknown)
#[no_mangle]
pub extern "C" fn process() -> i32 {
    // Plugin logic here
    42
}
```

## Available Plugins (planned)

| Plugin | Description |
|--------|-------------|
| auth-jwt | JWT authentication |
| auth-oauth | OAuth 2.0 |
| cache-redis | Redis-compatible cache |
| search-fts | Full-text search |
| analytics | Privacy-first analytics |
| payment-stripe | Stripe payments |
| email-smtp | Email sending |
| waf-shield | WAF + rate limiting |
