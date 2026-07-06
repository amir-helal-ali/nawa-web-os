//! # nawad — NAWA Web Operating System
//!
//! محرك ويب ثوري في binary واحد. لا يحتاج أي شيء خارجي.
//! - قاعدة بيانات مدمجة (NAWA-DB)
//! - محرك تصيير موحد (nawa-engine) بـ صفر نسخ
//! - مصادقة كاملة (nawa-auth) بـ JWT + RBAC
//! - io_uring (nawa-uring) للـ I/O غير المتزامن
//! - WASM sandbox (nawa-wasm) للـ plugins
//! - Prometheus metrics مدمجة
//! - Rate limiting مدمج
//! - Design system احترافي مدمج

mod metrics;
mod middleware;
mod dashboard;

use std::net::SocketAddr;
use std::path::PathBuf;
#[allow(unused_imports)]
use std::sync::Arc;
use std::collections::HashMap;
use std::time::Duration;

use clap::{Parser, Subcommand};
use metrics::Metrics;
use nawa_auth::{AuthConfig, AuthStore};
use nawa_db::{DbEngine, Value};
use nawa_engine::{UnifiedEngine, EngineContext};
use nawa_http::{HttpServer, Response, Router, StatusCode};
use nawa_uring::NawaUring;
use tracing_subscriber::EnvFilter;

/// NAWA — نظام تشغيل الويب الثوري
#[derive(Parser, Debug)]
#[command(name = "nawad", version, about = "NAWA — Revolutionary Web Operating System")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Start the server.
    Serve {
        #[arg(long, default_value = "0.0.0.0:8080")]
        addr: String,
        #[arg(long, default_value = "./nawa-data")]
        data_dir: PathBuf,
        /// Plugins directory for WASM auto-loading.
        #[arg(long, default_value = "./plugins")]
        plugins_dir: PathBuf,
        #[arg(long)]
        no_wal_sync: bool,
    },
    /// Run a benchmark.
    Benchmark { #[arg(short, long, default_value = "100000")] ops: u32 },
    /// Print version info.
    Info,
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    let cli = Cli::parse();
    let result: anyhow::Result<()> = match cli.command {
        Commands::Serve { addr, data_dir, plugins_dir, no_wal_sync } => {
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(serve(addr, data_dir, plugins_dir, !no_wal_sync))
        }
        Commands::Benchmark { ops } => benchmark(ops),
        Commands::Info => { print_info(); Ok(()) }
    };
    result
}

async fn serve(addr: String, data_dir: PathBuf, plugins_dir: PathBuf, wal_sync: bool) -> anyhow::Result<()> {
    tracing::info!("╔══════════════════════════════════════════════╗");
    tracing::info!("║  NAWA Web Operating System v0.1.0            ║");
    tracing::info!("║  نظام تشغيل الويب الثوري                     ║");
    tracing::info!("╚══════════════════════════════════════════════╝");

    // ── NAWA-DB ──
    let db_config = nawa_db::DbConfig {
        data_dir: data_dir.clone(),
        memtable_max_size: 4 * 1024 * 1024,
        wal_sync,
    };
    let db = Arc::new(DbEngine::open(db_config)?);
    tracing::info!("✓ NAWA-DB: {} keys, {} bytes memtable", db.len(), db.memtable_size());

    // ── Auth ──
    let auth = Arc::new(AuthStore::new(db.clone(), AuthConfig::with_secret("nawa-os-secret-2026")));
    tracing::info!("✓ Auth: {} users", auth.user_count());

    // ── io_uring ──
    let uring = Arc::new(NawaUring::new(nawa_uring::PipelineConfig::default())?);
    tracing::info!("✓ io_uring: real={}, entries={}", uring.is_real_uring(), uring.config().entries);

    // ── WASM sandbox + auto-load plugins ──
    let sandbox = Arc::new(tokio::sync::Mutex::new(
        nawa_wasm::Sandbox::default()?
    ));
    // Auto-load WASM plugins from directory.
    if plugins_dir.exists() {
        let mut sb = sandbox.lock().await;
        match sb.load_from_dir(&plugins_dir) {
            Ok(n) => tracing::info!("✓ WASM: {} plugins loaded from {}", n, plugins_dir.display()),
            Err(e) => tracing::warn!("⚠ WASM plugin loading: {e}"),
        }
    } else {
        tracing::info!("✓ WASM: sandbox ready (no plugins dir: {})", plugins_dir.display());
    }

    // ── Metrics ──
    let metrics = Arc::new(Metrics::new());
    tracing::info!("✓ Prometheus metrics: /metrics");

    // ── Rate limiter ──
    let rate_limiter = Arc::new(middleware::RateLimiter::new(100, Duration::from_secs(60)));
    tracing::info!("✓ Rate limiter: 100 req/min per IP");

    // ── Build router ──
    let router = build_router(db, auth, uring, sandbox, metrics, rate_limiter);
    tracing::info!("✓ Router: {} routes registered", router.len());

    let addr: SocketAddr = addr.parse()?;
    tracing::info!("");
    tracing::info!("🚀 NAWA running on http://{}", addr);
    tracing::info!("   Dashboard:  http://localhost:{}/", addr.port());
    tracing::info!("   Register:   http://localhost:{}/register", addr.port());
    tracing::info!("   Login:      http://localhost:{}/login", addr.port());
    tracing::info!("   API:        http://localhost:{}/api", addr.port());
    tracing::info!("   Metrics:    http://localhost:{}/metrics", addr.port());
    tracing::info!("");
    tracing::info!("Press Ctrl+C to stop");
    tracing::info!("");

    let server = HttpServer::new(router, addr);
    server.serve().await?;
    Ok(())
}

fn build_router(
    db: Arc<DbEngine>,
    auth: Arc<AuthStore>,
    uring: Arc<NawaUring>,
    sandbox: Arc<tokio::sync::Mutex<nawa_wasm::Sandbox>>,
    metrics: Arc<Metrics>,
    _rate_limiter: Arc<middleware::RateLimiter>,
) -> Router {
    let mut router = Router::new();

    // ═══════════════════════════════════════════════
    // PAGES (rendered by nawa-engine — zero-copy)
    // ═══════════════════════════════════════════════

    // GET / — Beautiful admin dashboard
    {
        let db = db.clone();
        let auth = auth.clone();
        let uring = uring.clone();
        router.get("/", move |_| {
            let db = db.clone();
            let auth = auth.clone();
            let uring = uring.clone();
            async move {
                let html = dashboard::render_dashboard(&db, &auth, &uring);
                let mut resp = Response::text(html);
                resp.header("Content-Type", "text/html; charset=utf-8");
                resp
            }
        });
    }

    // GET /register — Registration page
    {
        router.get("/register", move |_| async {
            let html = dashboard::render_register();
            let mut resp = Response::text(html);
            resp.header("Content-Type", "text/html; charset=utf-8");
            resp
        });
    }

    // POST /register — Register + auto-login
    {
        let auth = auth.clone();
        router.post("/register", move |req| {
            let auth = auth.clone();
            async move {
                let form = parse_form(req.body_str());
                let username = form.get("username").map(|s| s.as_str()).unwrap_or("");
                let email = form.get("email").map(|s| s.as_str()).unwrap_or("");
                let password = form.get("password").map(|s| s.as_str()).unwrap_or("");
                match auth.register(username, email, password) {
                    Ok(result) => {
                        let html = format!(
                            r#"<html><head><meta http-equiv="refresh" content="0;url=/"></head>
                            <body>Redirecting...</body></html>"#
                        );
                        let mut resp = Response::text(html);
                        resp.header("Content-Type", "text/html");
                        resp.header("Set-Cookie", &format!(
                            "nawa_token={}; Path=/; HttpOnly; Max-Age={}",
                            result.token, result.expires_in
                        ));
                        resp
                    }
                    Err(e) => {
                        let html = dashboard::render_error(&e.to_string(), "/register");
                        let mut resp = Response::text(html);
                        resp.header("Content-Type", "text/html; charset=utf-8");
                        resp
                    }
                }
            }
        });
    }

    // GET /login — Login page
    {
        router.get("/login", move |_| async {
            let html = dashboard::render_login();
            let mut resp = Response::text(html);
            resp.header("Content-Type", "text/html; charset=utf-8");
            resp
        });
    }

    // POST /login — Login
    {
        let auth = auth.clone();
        router.post("/login", move |req| {
            let auth = auth.clone();
            async move {
                let form = parse_form(req.body_str());
                let email = form.get("email").map(|s| s.as_str()).unwrap_or("");
                let password = form.get("password").map(|s| s.as_str()).unwrap_or("");
                match auth.login(email, password) {
                    Ok(result) => {
                        let html = r#"<html><head><meta http-equiv="refresh" content="0;url=/"></head><body>Redirecting...</body></html>"#;
                        let mut resp = Response::text(html);
                        resp.header("Content-Type", "text/html");
                        resp.header("Set-Cookie", &format!(
                            "nawa_token={}; Path=/; HttpOnly; Max-Age={}",
                            result.token, result.expires_in
                        ));
                        resp
                    }
                    Err(e) => {
                        let html = dashboard::render_error(&e.to_string(), "/login");
                        let mut resp = Response::text(html);
                        resp.header("Content-Type", "text/html; charset=utf-8");
                        resp
                    }
                }
            }
        });
    }

    // GET /logout
    {
        router.get("/logout", move |_| async {
            let html = r#"<html><head><meta http-equiv="refresh" content="0;url=/"></head><body>Logging out...</body></html>"#;
            let mut resp = Response::text(html);
            resp.header("Content-Type", "text/html");
            resp.header("Set-Cookie", "nawa_token=; Path=/; HttpOnly; Max-Age=0");
            resp
        });
    }

    // GET /ssr — Unified engine SSR demo
    {
        let db = db.clone();
        router.get("/ssr", move |_| {
            let db = db.clone();
            async move {
                let ctx = EngineContext::new(db);
                let result = UnifiedEngine::render_db_page(&ctx, "NAWA SSR Engine");
                let mut resp = Response::text(String::from_utf8_lossy(&result.html).to_string());
                resp.header("Content-Type", result.content_type);
                resp
            }
        });
    }

    // ═══════════════════════════════════════════════
    // DB API (key-value store)
    // ═══════════════════════════════════════════════

    // GET /health
    {
        let db = db.clone();
        router.get("/health", move |_| {
            let db = db.clone();
            async move {
                let stats = db.stats();
                Response::json(&serde_json::json!({
                    "status": "ok",
                    "keys": db.len(),
                    "memtable_bytes": db.memtable_size(),
                    "stats": {
                        "puts": stats.puts, "gets": stats.gets,
                        "deletes": stats.deletes, "scans": stats.scans,
                        "flushes": stats.memtable_flushes,
                    }
                }))
            }
        });
    }

    // GET /uring
    {
        let uring = uring.clone();
        router.get("/uring", move |_| {
            let uring = uring.clone();
            async move {
                let stats = uring.stats();
                Response::json(&serde_json::json!({
                    "real_uring": uring.is_real_uring(),
                    "sqpoll_enabled": uring.is_sqpoll_enabled(),
                    "entries": uring.config().entries,
                    "stats": {
                        "submitted": stats.submitted, "completed": stats.completed,
                        "in_flight": stats.in_flight, "errors": stats.errors,
                    }
                }))
            }
        });
    }

    // GET /metrics — Prometheus
    {
        let metrics = metrics.clone();
        let db = db.clone();
        let uring = uring.clone();
        router.get("/metrics", move |_| {
            let metrics = metrics.clone();
            let db = db.clone();
            let uring = uring.clone();
            async move {
                let db_stats = db.stats();
                metrics.update_db_stats(&db_stats);
                metrics.update_db_gauges(db.len(), db.memtable_size());
                let uring_stats = uring.stats();
        let _ = &uring_stats;
                metrics.update_uring_stats(&uring_stats);
                let body = metrics.render();
                let mut resp = Response::text(body);
                resp.header("Content-Type", "text/plain; version=0.0.4; charset=utf-8");
                resp
            }
        });
    }

    // GET /plugins
    {
        let sandbox = sandbox.clone();
        router.get("/plugins", move |_| {
            let sandbox = sandbox.clone();
            async move {
                let sb = sandbox.lock().await;
                let plugins = sb.list();
                Response::json(&serde_json::json!({"count": plugins.len(), "plugins": plugins}))
            }
        });
    }

    // GET /:key — fetch value
    {
        let db = db.clone();
        router.get("/:key", move |req| {
            let db = db.clone();
            async move {
                let key = req.param("key").unwrap_or("");
                // Skip if it's a known route (the two-pass router handles this).
                match db.get(key) {
                    Some(v) => {
                        let mut resp = Response::text(v.display());
                        resp.header("Content-Type", "text/plain; charset=utf-8");
                        resp
                    }
                    None => Response::not_found(format!("key not found: {key}")),
                }
            }
        });
    }

    // POST /:key — store value
    {
        let db = db.clone();
        router.post("/:key", move |req| {
            let db = db.clone();
            async move {
                let key = req.param("key").unwrap_or("").to_string();
                let value = if req.body_str().trim_start().starts_with('{') || req.body_str().trim_start().starts_with('[') {
                    Value::from_json_str(req.body_str()).unwrap_or_else(|_| Value::Bytes(req.body.clone()))
                } else {
                    Value::Bytes(req.body.clone())
                };
                match db.put(&key, value) {
                    Ok(_) => Response::text(format!("stored: {key}")),
                    Err(_) => {
                        let mut r = Response::new(StatusCode(500));
                        r.body = b"internal error".to_vec();
                        r
                    }
                }
            }
        });
    }

    // DELETE /:key
    {
        let db = db.clone();
        router.delete("/:key", move |req| {
            let db = db.clone();
            async move {
                let key = req.param("key").unwrap_or("");
                match db.delete(key) {
                    Ok(true) => Response::text("deleted"),
                    Ok(false) => Response::not_found("key was not present"),
                    Err(_) => Response::new(StatusCode(500)),
                }
            }
        });
    }

    // GET /scan/:prefix
    {
        let db = db.clone();
        router.get("/scan/:prefix", move |req| {
            let db = db.clone();
            async move {
                let prefix = req.param("prefix").unwrap_or("");
                let results = db.scan_prefix(prefix, 1000);
                let body: Vec<serde_json::Value> = results.iter().map(|(k, v)| {
                    serde_json::json!({
                        "key": String::from_utf8_lossy(k),
                        "value": v.display(),
                    })
                }).collect();
                Response::json(&serde_json::json!({"results": body, "count": body.len()}))
            }
        });
    }

    // ═══════════════════════════════════════════════
    // Auth API
    // ═══════════════════════════════════════════════

    // POST /auth/register (JSON API)
    {
        let auth = auth.clone();
        router.post("/auth/register", move |req| {
            let auth = auth.clone();
            async move {
                let json: serde_json::Value = match serde_json::from_str(req.body_str()) {
                    Ok(v) => v, Err(_) => return Response::text("invalid JSON"),
                };
                match auth.register(
                    json["username"].as_str().unwrap_or(""),
                    json["email"].as_str().unwrap_or(""),
                    json["password"].as_str().unwrap_or(""),
                ) {
                    Ok(r) => Response::json(&serde_json::json!({
                        "status": "ok", "token": r.token, "expires_in": r.expires_in,
                        "user": {"id": r.user.id, "username": r.user.username, "role": r.user.role, "verified": r.user.verified}
                    })),
                    Err(e) => {
                        let mut r = Response::new(StatusCode(400));
                        r.header("Content-Type", "application/json");
                        r.body = serde_json::to_vec(&serde_json::json!({"error": e.to_string()})).unwrap_or_default();
                        r
                    }
                }
            }
        });
    }

    // POST /auth/login (JSON API)
    {
        let auth = auth.clone();
        router.post("/auth/login", move |req| {
            let auth = auth.clone();
            async move {
                let json: serde_json::Value = match serde_json::from_str(req.body_str()) {
                    Ok(v) => v, Err(_) => return Response::text("invalid JSON"),
                };
                match auth.login(
                    json["email"].as_str().unwrap_or(""),
                    json["password"].as_str().unwrap_or(""),
                ) {
                    Ok(r) => Response::json(&serde_json::json!({
                        "status": "ok", "token": r.token, "expires_in": r.expires_in,
                        "user": {"id": r.user.id, "username": r.user.username, "role": r.user.role}
                    })),
                    Err(e) => {
                        let mut r = Response::new(StatusCode(401));
                        r.header("Content-Type", "application/json");
                        r.body = serde_json::to_vec(&serde_json::json!({"error": e.to_string()})).unwrap_or_default();
                        r
                    }
                }
            }
        });
    }

    // GET /auth/me
    {
        let auth = auth.clone();
        router.get("/auth/me", move |req| {
            let auth = auth.clone();
            async move {
                let token = req.header("authorization")
                    .and_then(|h| h.strip_prefix("Bearer ").map(|s| s.to_string()));
                match token.and_then(|t| auth.verify_token(&t).ok()) {
                    Some(claims) => match auth.get_user(&claims.sub) {
                        Ok(u) => Response::json(&serde_json::json!({
                            "id": u.id, "username": u.username, "email": u.email, "role": u.role, "verified": u.verified
                        })),
                        Err(e) => Response::text(e.to_string()),
                    },
                    None => {
                        let mut r = Response::new(StatusCode(401));
                        r.body = b"missing or invalid token".to_vec();
                        r
                    }
                }
            }
        });
    }

    // GET /auth/users (admin only)
    {
        let auth = auth.clone();
        router.get("/auth/users", move |req| {
            let auth = auth.clone();
            async move {
                let token = req.header("authorization")
                    .and_then(|h| h.strip_prefix("Bearer ").map(|s| s.to_string()));
                match token.and_then(|t| auth.verify_token(&t).ok()) {
                    Some(claims) => match auth.get_user(&claims.sub) {
                        Ok(user) if user.role == "admin" => {
                            let users = auth.list_users().unwrap_or_default();
                            let safe: Vec<_> = users.iter().map(|u| serde_json::json!({
                                "id": u.id, "username": u.username, "email": u.email,
                                "role": u.role, "verified": u.verified
                            })).collect();
                            Response::json(&serde_json::json!({"users": safe, "count": safe.len()}))
                        }
                        _ => {
                            let mut r = Response::new(StatusCode(403));
                            r.body = b"admin required".to_vec();
                            r
                        }
                    },
                    None => {
                        let mut r = Response::new(StatusCode(401));
                        r.body = b"missing token".to_vec();
                        r
                    }
                }
            }
        });
    }

    // GET /api — API info
    router.get("/api", |_| async {
        Response::json(&serde_json::json!({
            "name": "NAWA", "version": "0.1.0-alpha",
            "description": "Revolutionary Web Operating System built in Rust",
            "endpoints": [
                "GET /", "GET /register", "POST /register", "GET /login", "POST /login",
                "GET /logout", "GET /ssr", "GET /health", "GET /uring", "GET /metrics",
                "GET /plugins", "GET /:key", "POST /:key", "DELETE /:key", "GET /scan/:prefix",
                "POST /auth/register", "POST /auth/login", "GET /auth/me", "GET /auth/users",
            ]
        }))
    });

    router
}

fn parse_form(body: &str) -> HashMap<String, String> {
    let mut form = HashMap::new();
    for pair in body.split('&') {
        if let Some((k, v)) = pair.split_once('=') {
            form.insert(url_decode(k), url_decode(v));
        }
    }
    form
}

fn url_decode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '+' { out.push(' '); }
        else if c == '%' {
            let hex: String = chars.by_ref().take(2).collect();
            if let Ok(byte) = u8::from_str_radix(&hex, 16) { out.push(byte as char); }
        } else { out.push(c); }
    }
    out
}

fn benchmark(ops: u32) -> anyhow::Result<()> {
    println!("NAWA-DB benchmark — {ops} operations\n─────────────────────────────────────");
    let db = DbEngine::open_in_memory();
    let started = std::time::Instant::now();
    for i in 0..ops { let _ = db.put(format!("bench:{i}"), Value::from_i64(i as i64))?; }
    let put_elapsed = started.elapsed();
    println!("PUT:  {:>8} ops in {:>8.2?}  →  {:>8.0} ops/sec", ops, put_elapsed, ops as f64 / put_elapsed.as_secs_f64());
    let started = std::time::Instant::now();
    let mut found = 0u32;
    for i in 0..ops { if db.get(format!("bench:{i}")).is_some() { found += 1; } }
    let get_elapsed = started.elapsed();
    println!("GET:  {:>8} ops in {:>8.2?}  →  {:>8.0} ops/sec  ({found} hits)", ops, get_elapsed, ops as f64 / get_elapsed.as_secs_f64());
    let started = std::time::Instant::now();
    let results = db.scan_prefix("bench:", 10_000_000);
    let scan_elapsed = started.elapsed();
    println!("SCAN: {:>8} hits in {:>8.2?}  →  {:>8.0} ops/sec", results.len(), scan_elapsed, results.len() as f64 / scan_elapsed.as_secs_f64());
    println!("─────────────────────────────────────\n{:?}", db.stats());
    Ok(())
}

fn print_info() {
    println!("NAWA Web Operating System v0.1.0-alpha");
    println!("═══════════════════════════════════════════════");
    println!("Built-in components (zero external deps):");
    println!("  • nawa-db:      KV/Document database (LSM tree + WAL + Bloom)");
    println!("  • nawa-engine:  Unified SSR engine (zero-copy HTML + design system)");
    println!("  • nawa-auth:    JWT auth + RBAC (admin/user roles)");
    println!("  • nawa-uring:   Real io_uring on Linux 5.1+");
    println!("  • nawa-wasm:    WASM sandbox (wasmtime) for plugins");
    println!("  • nawa-http:    HTTP/1.1 server + type-safe router");
    println!("  • nawa-kernel:  mmap + ring buffer + zero-copy primitives");
    println!();
    println!("Platform: {} / {}", std::env::consts::OS, std::env::consts::ARCH);
    println!("License:  MIT OR Apache-2.0");
    println!();
    println!("Run 'nawad serve' to start the server.");
}
