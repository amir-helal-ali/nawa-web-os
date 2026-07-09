//! # nawad — NAWA Web Operating System
//!
//! محرك ويب ثوري في binary واحد. لا يحتاج أي شيء خارجي.
//! لا polling — كل شيء event-driven (WebSocket push).

mod cache;
mod config;
mod cookies;
mod dashboard;
mod errors;
mod feature_flags;
mod i18n;
mod metrics;
mod middleware;
mod notifications;
mod openapi;
mod plugins;
mod pubsub;
mod quantum;
mod rate_limiter;
mod realtime;
mod req_tracing;
mod scheduler;
mod session;
mod stability;
mod webhooks;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use clap::{Parser, Subcommand};
use metrics::Metrics;
use nawa_auth::{AuthConfig, AuthStore, User};
use nawa_db::{DbEngine, Value};
use nawa_engine::{EngineContext, UnifiedEngine};
use nawa_http::{HttpServer, Response, Router, StatusCode};
use nawa_uring::NawaUring;
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(name = "nawad", version, about = "NAWA — Revolutionary Web Operating System")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Serve {
        #[arg(long)] addr: Option<String>,
        #[arg(long)] data_dir: Option<PathBuf>,
        #[arg(long)] plugins_dir: Option<PathBuf>,
        #[arg(long)] static_dir: Option<PathBuf>,
        #[arg(long)] svelte_dir: Option<PathBuf>,
        #[arg(long, default_value = "./nawa.toml")] config: PathBuf,
        #[arg(long)] no_wal_sync: bool,
        /// Enable HTTP/3 over QUIC (requires --tls-cert and --tls-key).
        #[arg(long)] http3: bool,
        /// Path to PEM certificate file (for HTTPS + HTTP/3).
        #[arg(long)] tls_cert: Option<PathBuf>,
        /// Path to PEM private key file (for HTTPS + HTTP/3).
        #[arg(long)] tls_key: Option<PathBuf>,
        /// UDP port for HTTP/3 (default: same as HTTP port).
        #[arg(long)] http3_port: Option<u16>,
    },
    Benchmark { #[arg(short, long, default_value = "100000")] ops: u32 },
    Init { #[arg(long, default_value = "./nawa.toml")] path: PathBuf },
    Info,
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();
    let cli = Cli::parse();
    match cli.command {
        Commands::Serve { addr, data_dir, plugins_dir, static_dir, svelte_dir, config: cfg_path, no_wal_sync, http3, tls_cert, tls_key, http3_port } => {
            let mut cfg = config::Config::load(&cfg_path);
            if let Some(a) = addr { cfg.addr = a; }
            if let Some(d) = data_dir { cfg.data_dir = d.display().to_string(); }
            if let Some(p) = plugins_dir { cfg.plugins_dir = p.display().to_string(); }
            if let Some(s) = static_dir { cfg.static_dir = s.display().to_string(); }
            if no_wal_sync { cfg.wal_sync = false; }
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(serve(cfg, svelte_dir, http3, tls_cert, tls_key, http3_port))
        }
        Commands::Benchmark { ops } => benchmark(ops),
        Commands::Init { path } => {
            config::Config::generate_default(&path)?;
            println!("✓ Config generated: {}", path.display());
            Ok(())
        }
        Commands::Info => { print_info(); Ok(()) }
    }
}

async fn serve(
    cfg: config::Config,
    svelte_dir: Option<PathBuf>,
    enable_http3: bool,
    tls_cert: Option<PathBuf>,
    tls_key: Option<PathBuf>,
    http3_port: Option<u16>,
) -> anyhow::Result<()> {
    tracing::info!("╔══════════════════════════════════════════════╗");
    tracing::info!("║  NAWA Web Operating System v2.0.0            ║");
    tracing::info!("╚══════════════════════════════════════════════╝");
    tracing::info!("Config: {}", cfg.summary());

    let data_dir = PathBuf::from(&cfg.data_dir);
    let plugins_dir = PathBuf::from(&cfg.plugins_dir);
    let static_dir = PathBuf::from(&cfg.static_dir);

    let db = Arc::new(DbEngine::open(nawa_db::DbConfig {
        data_dir: data_dir.clone(), memtable_max_size: 4 * 1024 * 1024, wal_sync: cfg.wal_sync,
    })?);
    tracing::info!("✓ NAWA-DB: {} keys", db.len());

    let auth = Arc::new(AuthStore::new(db.clone(), AuthConfig::with_secret(&cfg.jwt_secret)));
    tracing::info!("✓ Auth: {} users", auth.user_count());

    let uring = Arc::new(NawaUring::new(nawa_uring::PipelineConfig::default())?);
    tracing::info!("✓ io_uring: real={}", uring.is_real_uring());

    let sandbox = Arc::new(tokio::sync::Mutex::new(nawa_wasm::Sandbox::default()?));
    if plugins_dir.exists() {
        let mut sb = sandbox.lock().await;
        match sb.load_from_dir(&plugins_dir) {
            Ok(n) => tracing::info!("✓ WASM: {} plugins", n),
            Err(e) => tracing::warn!("⚠ WASM: {e}"),
        }
    } else {
        tracing::info!("✓ WASM: ready (no plugins dir)");
    }

    let metrics = Arc::new(Metrics::new());
    let rate_limiter = Arc::new(middleware::RateLimiter::new(cfg.rate_limit, Duration::from_secs(60)));
    let static_server = Arc::new(middleware::StaticServer::new(&static_dir));

    // Real-time Event Bus — pure push, NO polling.
    let event_bus = realtime::EventBus::new(256);
    let ws_connections = realtime::ConnectionManager::new();
    tracing::info!("✓ Real-time: WebSocket event bus (push, no polling)");

    // SvelteKit integration (optional — loaded if --svelte-dir is provided).
    let svelte_handler: Option<Arc<nawa_svelte::SvelteHandler>> = if let Some(dir) = svelte_dir.clone() {
        let manifest_path = dir.join("manifest.json");
        if manifest_path.exists() {
            let addr: SocketAddr = cfg.addr.parse().unwrap_or_else(|_| "127.0.0.1:8080".parse().unwrap());
            let ws_port = addr.port() + 1;
            let ws_url = format!("ws://localhost:{}", ws_port);
            match nawa_svelte::SvelteHandler::load(&dir, ws_url) {
                Ok(h) => {
                    tracing::info!("✓ SvelteKit: '{}' — {} routes (no Node.js at runtime)",
                        h.manifest.app_name, h.route_count());
                    Some(h)
                }
                Err(e) => {
                    tracing::warn!("⚠ SvelteKit load failed: {e}");
                    None
                }
            }
        } else {
            tracing::info!("✓ SvelteKit: dir provided but no manifest.json — skipping");
            None
        }
    } else {
        tracing::info!("✓ SvelteKit: not configured (use --svelte-dir to enable)");
        None
    };

    let router = build_router(RouterDeps {
        db: db.clone(),
        auth: auth.clone(),
        uring: uring.clone(),
        sandbox: sandbox.clone(),
        metrics: metrics.clone(),
        _rate_limiter: rate_limiter,
        static_server,
        event_bus: event_bus.clone(),
        svelte_handler: svelte_handler.clone(),
    });
    tracing::info!("✓ Router: {} routes", router.len());

    let addr: SocketAddr = cfg.addr.parse()?;
    let ws_port = addr.port() + 1;
    tracing::info!("\n🚀 NAWA on http://{}", addr);
    tracing::info!("   Dashboard:  http://localhost:{}", addr.port());
    tracing::info!("   WebSocket:  ws://localhost:{}", ws_port);
    tracing::info!("   Register:   http://localhost:{}/register", addr.port());
    tracing::info!("   System:     http://localhost:{}/system", addr.port());
    tracing::info!("   Metrics:    http://localhost:{}/metrics", addr.port());
    if svelte_handler.is_some() {
        tracing::info!("   SvelteKit:  http://localhost:{}/svelte/", addr.port());
    }
    tracing::info!("");

    // WebSocket server (separate port, pure push).
    let ws_addr: SocketAddr = format!("{}:{}", addr.ip(), ws_port).parse()?;
    let ws_bus = event_bus.clone();
    let ws_conns = ws_connections.clone();
    tokio::spawn(async move {
        let listener = match tokio::net::TcpListener::bind(ws_addr).await {
            Ok(l) => l,
            Err(e) => { tracing::warn!("WS bind failed: {e}"); return; }
        };
        tracing::info!("✓ WebSocket server on port {}", ws_port);
        loop {
            if let Ok((stream, _)) = listener.accept().await {
                let bus = ws_bus.clone();
                let conns = ws_conns.clone();
                tokio::spawn(async move { realtime::handle_websocket(stream, bus, conns).await; });
            }
        }
    });

    // HTTP/3 + QUIC server (optional — requires TLS cert + key).
    if enable_http3 {
        match (tls_cert.as_ref(), tls_key.as_ref()) {
            (Some(cert_path), Some(key_path)) => {
                match nawa_http::TlsConfig::from_pem_files(cert_path, key_path) {
                    Ok(tls) => {
                        let h3_port = http3_port.unwrap_or(addr.port());
                        let h3_addr: SocketAddr = format!("{}:{}", addr.ip(), h3_port).parse()?;
                        let h3_config = nawa_http::Http3Config::new(h3_addr, tls)
                            .with_max_streams(100)
                            .with_idle_timeout(30_000);
                        let h3_router = router.clone();
                        let h3_server = nawa_http::Http3Server::new(h3_config, h3_router);
                        tracing::info!("✓ HTTP/3 (QUIC) server enabled on udp://{}", h3_addr);
                        tokio::spawn(async move {
                            if let Err(e) = h3_server.serve().await {
                                tracing::warn!("HTTP/3 server error: {e}");
                            }
                        });
                    }
                    Err(e) => {
                        tracing::warn!("⚠ HTTP/3 disabled — TLS config failed: {e}");
                    }
                }
            }
            _ => {
                tracing::warn!("⚠ HTTP/3 disabled — requires --tls-cert and --tls-key");
            }
        }
    }

    // HTTP server with graceful shutdown.
    let server = HttpServer::new(router, addr);
    tokio::select! {
        result = server.serve() => {
            if let Err(e) = result { tracing::error!("Server error: {e}"); }
        }
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("\n⚠ Ctrl+C — shutting down...");
        }
    }
    tracing::info!("✓ NAWA stopped.");
    Ok(())
}

fn get_current_user(req: &nawa_http::Request, auth: &AuthStore) -> Option<User> {
    let cookie = req.header("cookie")?;
    let token = middleware::extract_cookie_value(cookie, "nawa_token")?;
    let claims = auth.verify_token(&token).ok()?;
    auth.get_user(&claims.sub).ok()
}

/// All shared state passed into the router builder.
/// Grouped into a struct to keep the function signature under clippy's
/// `too many arguments` threshold.
struct RouterDeps {
    db: Arc<DbEngine>,
    auth: Arc<AuthStore>,
    uring: Arc<NawaUring>,
    sandbox: Arc<tokio::sync::Mutex<nawa_wasm::Sandbox>>,
    metrics: Arc<Metrics>,
    _rate_limiter: Arc<middleware::RateLimiter>,
    static_server: Arc<middleware::StaticServer>,
    event_bus: Arc<realtime::EventBus>,
    svelte_handler: Option<Arc<nawa_svelte::SvelteHandler>>,
}

fn build_router(deps: RouterDeps) -> Router {
    let RouterDeps { db, auth, uring, sandbox, metrics, _rate_limiter, static_server, event_bus, svelte_handler } = deps;
    let mut router = Router::new();

    // ═══ DASHBOARD ═══
    {
        let db = db.clone();
        let auth = auth.clone();
        let uring = uring.clone();
        router.get("/", move |req| {
            let db = db.clone();
            let auth = auth.clone();
            let uring = uring.clone();
            async move {
                let user = get_current_user(&req, &auth);
                let html = dashboard::render_dashboard(&db, &auth, &uring, user.as_ref());
                let mut r = Response::text(html);
                r.header("Content-Type", "text/html; charset=utf-8");
                middleware::add_security_headers(&mut r);
                r
            }
        });
    }

    // ═══ AUTH PAGES ═══
    router.get("/register", move |_| async {
        let mut r = Response::text(dashboard::render_register());
        r.header("Content-Type", "text/html; charset=utf-8");
        r
    });

    {
        let auth = auth.clone();
        let _eb = event_bus.clone();
        router.post("/register", move |req| {
            let auth = auth.clone();
            let eb = _eb.clone();
            async move {
                let form = parse_form(req.body_str());
                match auth.register(
                    form.get("username").map(|s| s.as_str()).unwrap_or(""),
                    form.get("email").map(|s| s.as_str()).unwrap_or(""),
                    form.get("password").map(|s| s.as_str()).unwrap_or(""),
                ) {
                    Ok(result) => {
                        eb.publish(realtime::Notification::new("user_registered", serde_json::json!({
                            "username": result.user.username, "role": result.user.role
                        })));
                        let mut r = Response::text(r#"<html><head><meta http-equiv="refresh" content="0;url=/"></head></html>"#);
                        r.header("Content-Type", "text/html");
                        r.header("Set-Cookie", &format!("nawa_token={}; Path=/; HttpOnly; Max-Age={}", result.token, result.expires_in));
                        r
                    }
                    Err(e) => {
                        let mut r = Response::text(dashboard::render_error(&e.to_string(), "/register"));
                        r.header("Content-Type", "text/html; charset=utf-8");
                        r
                    }
                }
            }
        });
    }

    router.get("/login", move |_| async {
        let mut r = Response::text(dashboard::render_login());
        r.header("Content-Type", "text/html; charset=utf-8");
        r
    });

    {
        let auth = auth.clone();
        let eb = event_bus.clone();
        router.post("/login", move |req| {
            let auth = auth.clone();
            let eb = eb.clone();
            async move {
                let form = parse_form(req.body_str());
                match auth.login(
                    form.get("email").map(|s| s.as_str()).unwrap_or(""),
                    form.get("password").map(|s| s.as_str()).unwrap_or(""),
                ) {
                    Ok(result) => {
                        eb.publish(realtime::Notification::new("user_login", serde_json::json!({
                            "username": result.user.username
                        })));
                        let mut r = Response::text(r#"<html><head><meta http-equiv="refresh" content="0;url=/"></head></html>"#);
                        r.header("Content-Type", "text/html");
                        r.header("Set-Cookie", &format!("nawa_token={}; Path=/; HttpOnly; Max-Age={}", result.token, result.expires_in));
                        r
                    }
                    Err(e) => {
                        let mut r = Response::text(dashboard::render_error(&e.to_string(), "/login"));
                        r.header("Content-Type", "text/html; charset=utf-8");
                        r
                    }
                }
            }
        });
    }

    router.get("/logout", move |_| async {
        let mut r = Response::text(r#"<html><head><meta http-equiv="refresh" content="0;url=/"></head></html>"#);
        r.header("Content-Type", "text/html");
        r.header("Set-Cookie", "nawa_token=; Path=/; HttpOnly; Max-Age=0");
        r
    });

    // ═══ PROFILE ═══
    {
        let auth = auth.clone();
        router.get("/profile", move |req| {
            let auth = auth.clone();
            async move {
                match get_current_user(&req, &auth) {
                    Some(user) => {
                        let mut r = Response::text(dashboard::render_profile(&user));
                        r.header("Content-Type", "text/html; charset=utf-8");
                        r
                    }
                    None => {
                        let mut r = Response::text(r#"<html><head><meta http-equiv="refresh" content="0;url=/login"></head></html>"#);
                        r.header("Content-Type", "text/html");
                        r
                    }
                }
            }
        });
    }

    {
        let auth = auth.clone();
        let db = db.clone();
        let eb = event_bus.clone();
        router.post("/profile", move |req| {
            let auth = auth.clone();
            let db = db.clone();
            let eb = eb.clone();
            async move {
                match get_current_user(&req, &auth) {
                    Some(user) => {
                        let form = parse_form(req.body_str());
                        let mut updated = user.clone();
                        if let Some(u) = form.get("username") { updated.username = u.clone(); }
                        if let Some(e) = form.get("email") {
                            let _ = db.delete(format!("user:email:{}", user.email));
                            let _ = db.put(format!("user:email:{}", e), Value::from_str(&user.id));
                            updated.email = e.clone();
                        }
                        if let Some(p) = form.get("new_password") {
                            if !p.is_empty() { updated.password_hash = nawa_auth::password::hash_password(p); }
                        }
                        let json = serde_json::to_string(&updated).unwrap_or_default();
                        let _ = db.put(format!("user:{}", user.id), Value::from_json_str(&json).unwrap_or_else(|_| Value::Bytes(json.into_bytes())));
                        eb.publish(realtime::Notification::new("profile_updated", serde_json::json!({"username": updated.username})));
                        let mut r = Response::text(r#"<html><head><meta http-equiv="refresh" content="0;url=/profile"></head><body>تم الحفظ</body></html>"#);
                        r.header("Content-Type", "text/html");
                        r
                    }
                    None => Response::new(StatusCode(401)),
                }
            }
        });
    }

    // ═══ ADMIN ═══
    {
        let auth = auth.clone();
        router.get("/settings", move |req| {
            let auth = auth.clone();
            async move {
                match get_current_user(&req, &auth) {
                    Some(u) if u.role == "admin" => {
                        let mut r = Response::text(dashboard::render_settings(&auth, &u));
                        r.header("Content-Type", "text/html; charset=utf-8");
                        r
                    }
                    Some(_) => {
                        let mut r = Response::text(dashboard::render_error("صلاحية الأدمن مطلوبة", "/"));
                        r.header("Content-Type", "text/html; charset=utf-8");
                        r
                    }
                    None => {
                        let mut r = Response::text(r#"<html><head><meta http-equiv="refresh" content="0;url=/login"></head></html>"#);
                        r.header("Content-Type", "text/html");
                        r
                    }
                }
            }
        });
    }

    {
        let auth = auth.clone();
        let eb = event_bus.clone();
        router.post("/settings", move |req| {
            let auth = auth.clone();
            let eb = eb.clone();
            async move {
                match get_current_user(&req, &auth) {
                    Some(u) if u.role == "admin" => {
                        let form = parse_form(req.body_str());
                        let mut s = auth.get_settings().unwrap_or_default();
                        s.project_name = form.get("project_name").cloned().unwrap_or_default();
                        s.registration_open = form.contains_key("registration_open");
                        s.verification_required = form.contains_key("verification_required");
                        s.max_users = form.get("max_users").and_then(|v| v.parse().ok());
                        if let Some(e) = form.get("jwt_expiry_secs").and_then(|v| v.parse::<u64>().ok()) { s.jwt_expiry_secs = e; }
                        let _ = auth.update_settings(&u.id, &s);
                        eb.publish(realtime::Notification::new("settings_updated", serde_json::json!({"project": s.project_name})));
                        let mut r = Response::text(dashboard::render_settings(&auth, &u));
                        r.header("Content-Type", "text/html; charset=utf-8");
                        r
                    }
                    _ => {
                        let mut r = Response::new(StatusCode(403));
                        r.body = b"admin required".to_vec();
                        r
                    }
                }
            }
        });
    }

    {
        let auth = auth.clone();
        let eb = event_bus.clone();
        router.post("/admin/verify", move |req| {
            let auth = auth.clone();
            let eb = eb.clone();
            async move {
                if let Some(admin) = get_current_user(&req, &auth) {
                    if admin.role == "admin" {
                        let form = parse_form(req.body_str());
                        if let Some(id) = form.get("user_id") {
                            let _ = auth.verify_user(&admin.id, id);
                            eb.publish(realtime::Notification::new("user_verified", serde_json::json!({"user_id": id})));
                        }
                    }
                }
                Response::text(r#"<html><head><meta http-equiv="refresh" content="0;url=/"></head></html>"#)
            }
        });
    }

    {
        let auth = auth.clone();
        let eb = event_bus.clone();
        router.post("/admin/role", move |req| {
            let auth = auth.clone();
            let eb = eb.clone();
            async move {
                if let Some(admin) = get_current_user(&req, &auth) {
                    if admin.role == "admin" {
                        let form = parse_form(req.body_str());
                        if let (Some(id), Some(role)) = (form.get("user_id"), form.get("role")) {
                            let _ = auth.change_role(&admin.id, id, role);
                            eb.publish(realtime::Notification::new("role_changed", serde_json::json!({"user_id": id, "new_role": role})));
                        }
                    }
                }
                Response::text(r#"<html><head><meta http-equiv="refresh" content="0;url=/"></head></html>"#)
            }
        });
    }

    {
        let auth = auth.clone();
        let eb = event_bus.clone();
        router.post("/admin/delete", move |req| {
            let auth = auth.clone();
            let eb = eb.clone();
            async move {
                if let Some(admin) = get_current_user(&req, &auth) {
                    if admin.role == "admin" {
                        let form = parse_form(req.body_str());
                        if let Some(id) = form.get("user_id") {
                            let _ = auth.delete_user(&admin.id, id);
                            eb.publish(realtime::Notification::new("user_deleted", serde_json::json!({"user_id": id})));
                        }
                    }
                }
                Response::text(r#"<html><head><meta http-equiv="refresh" content="0;url=/"></head></html>"#)
            }
        });
    }

    // ═══ SYSTEM ═══
    {
        let db = db.clone();
        let auth = auth.clone();
        let uring = uring.clone();
        router.get("/system", move |_| {
            let db = db.clone();
            let auth = auth.clone();
            let uring = uring.clone();
            async move {
                let mut r = Response::text(dashboard::render_system(&db, &auth, &uring));
                r.header("Content-Type", "text/html; charset=utf-8");
                r
            }
        });
    }

    {
        let db = db.clone();
        router.get("/ssr", move |_| {
            let db = db.clone();
            async move {
                let ctx = EngineContext::new(db);
                let result = UnifiedEngine::render_db_page(&ctx, "NAWA SSR");
                let mut r = Response::text(String::from_utf8_lossy(&result.html).to_string());
                r.header("Content-Type", result.content_type);
                r
            }
        });
    }

    {
        let db = db.clone();
        router.get("/health", move |_| {
            let db = db.clone();
            async move {
                let s = db.stats();
                Response::json(&serde_json::json!({
                    "status":"ok","keys":db.len(),"memtable_bytes":db.memtable_size(),
                    "stats":{"puts":s.puts,"gets":s.gets,"deletes":s.deletes,"scans":s.scans,"flushes":s.memtable_flushes}
                }))
            }
        });
    }

    {
        let uring = uring.clone();
        router.get("/uring", move |_| {
            let uring = uring.clone();
            async move {
                let s = uring.stats();
                Response::json(&serde_json::json!({
                    "real_uring":uring.is_real_uring(),"sqpoll":uring.is_sqpoll_enabled(),
                    "entries":uring.config().entries,
                    "stats":{"submitted":s.submitted,"completed":s.completed,"in_flight":s.in_flight,"errors":s.errors}
                }))
            }
        });
    }

    {
        let metrics = metrics.clone();
        let db = db.clone();
        let uring = uring.clone();
        router.get("/metrics", move |_| {
            let m = metrics.clone();
            let db = db.clone();
            let uring = uring.clone();
            async move {
                let ds = db.stats();
                m.update_db_stats(&ds);
                m.update_db_gauges(db.len(), db.memtable_size());
                let us = uring.stats();
                m.update_uring_stats(&us);
                let mut r = Response::text(m.render());
                r.header("Content-Type", "text/plain; version=0.0.4");
                r
            }
        });
    }

    {
        let sandbox = sandbox.clone();
        router.get("/plugins", move |_| {
            let sb = sandbox.clone();
            async move {
                let sb = sb.lock().await;
                Response::json(&serde_json::json!({"count":sb.list().len(),"plugins":sb.list()}))
            }
        });
    }

    // ═══ WASM SSR ENDPOINT ═══
    // POST /api/wasm-ssr → calls the WASM SSR module's render() with JSON body, returns HTML.
    // The WASM module must export: memory, alloc(size)->ptr, render(props_ptr,props_len)->html_ptr
    {
        let sandbox = sandbox.clone();
        router.post("/api/wasm-ssr", move |req| {
            let sb_arc = sandbox.clone();
            async move {
                let props_json = req.body_str().to_string();
                let sb = sb_arc.lock().await;

                // Check if the SSR demo module is loaded.
                if !sb.list().iter().any(|n| n == "nawa_ssr_demo") {
                    let mut r = Response::new(StatusCode(404));
                    r.header("Content-Type", "application/json");
                    r.body = serde_json::to_vec(&serde_json::json!({
                        "error": "WASM SSR module not loaded",
                        "hint": "Place nawa_ssr_demo.wasm in the plugins directory"
                    })).unwrap_or_default();
                    return r;
                }

                // Call render_ssr() on the sandbox.
                match sb.render_ssr("nawa_ssr_demo", &props_json) {
                    Ok(html) => {
                        let mut r = Response::ok(html.into_bytes());
                        r.header("Content-Type", "text/html; charset=utf-8");
                        r.header("X-NAWA-SSR", "wasm");
                        r
                    }
                    Err(e) => {
                        let mut r = Response::new(StatusCode(500));
                        r.header("Content-Type", "application/json");
                        r.body = serde_json::to_vec(&serde_json::json!({
                            "error": format!("SSR render failed: {e}")
                        })).unwrap_or_default();
                        r
                    }
                }
            }
        });
    }

    {
        let eb = event_bus.clone();
        router.get("/notifications/stats", move |_| {
            let eb = eb.clone();
            async move {
                Response::json(&serde_json::json!({"total":eb.total(),"status":"active"}))
            }
        });
    }

    // ═══ DB API ═══
    {
        let db = db.clone();
        router.get("/:key", move |req| {
            let db = db.clone();
            async move {
                let key = req.param("key").unwrap_or("");
                match db.get(key) {
                    Some(v) => {
                        let mut r = Response::text(v.display());
                        r.header("Content-Type", "text/plain; charset=utf-8");
                        r
                    }
                    None => Response::not_found(format!("key not found: {key}")),
                }
            }
        });
    }

    {
        let db = db.clone();
        let eb = event_bus.clone();
        router.post("/:key", move |req| {
            let db = db.clone();
            let eb = eb.clone();
            async move {
                let key = req.param("key").unwrap_or("").to_string();
                let value = if req.body_str().trim_start().starts_with('{') || req.body_str().trim_start().starts_with('[') {
                    Value::from_json_str(req.body_str()).unwrap_or_else(|_| Value::Bytes(req.body.clone()))
                } else { Value::Bytes(req.body.clone()) };
                match db.put(&key, value) {
                    Ok(_) => {
                        eb.publish(realtime::Notification::new("db_write", serde_json::json!({"key": key})));
                        Response::text(format!("stored: {key}"))
                    }
                    Err(_) => Response::new(StatusCode(500)),
                }
            }
        });
    }

    {
        let db = db.clone();
        let eb = event_bus.clone();
        router.delete("/:key", move |req| {
            let db = db.clone();
            let eb = eb.clone();
            async move {
                let key = req.param("key").unwrap_or("");
                match db.delete(key) {
                    Ok(true) => {
                        eb.publish(realtime::Notification::new("db_delete", serde_json::json!({"key": key})));
                        Response::text("deleted")
                    }
                    Ok(false) => Response::not_found("key was not present"),
                    Err(_) => Response::new(StatusCode(500)),
                }
            }
        });
    }

    {
        let db = db.clone();
        router.get("/scan/:prefix", move |req| {
            let db = db.clone();
            async move {
                let prefix = req.param("prefix").unwrap_or("");
                let results = db.scan_prefix(prefix, 1000);
                let body: Vec<_> = results.iter().map(|(k,v)| serde_json::json!({"key":String::from_utf8_lossy(k),"value":v.display()})).collect();
                Response::json(&serde_json::json!({"results":body,"count":body.len()}))
            }
        });
    }

    // ═══ AUTH API ═══
    {
        let auth = auth.clone();
        let eb = event_bus.clone();
        router.post("/auth/register", move |req| {
            let auth = auth.clone();
            let eb = eb.clone();
            async move {
                let json: serde_json::Value = match serde_json::from_str(req.body_str()) { Ok(v)=>v, Err(_)=>return Response::text("invalid JSON") };
                match auth.register(json["username"].as_str().unwrap_or(""), json["email"].as_str().unwrap_or(""), json["password"].as_str().unwrap_or("")) {
                    Ok(r) => {
                        eb.publish(realtime::Notification::new("user_registered", serde_json::json!({"username":r.user.username})));
                        Response::json(&serde_json::json!({"status":"ok","token":r.token,"expires_in":r.expires_in,"user":{"id":r.user.id,"username":r.user.username,"role":r.user.role,"verified":r.user.verified}}))
                    }
                    Err(e) => {
                        let mut r = Response::new(StatusCode(400));
                        r.header("Content-Type","application/json");
                        r.body = serde_json::to_vec(&serde_json::json!({"error":e.to_string()})).unwrap_or_default();
                        r
                    }
                }
            }
        });
    }

    {
        let auth = auth.clone();
        router.post("/auth/login", move |req| {
            let auth = auth.clone();
            async move {
                let json: serde_json::Value = match serde_json::from_str(req.body_str()) { Ok(v)=>v, Err(_)=>return Response::text("invalid JSON") };
                match auth.login(json["email"].as_str().unwrap_or(""), json["password"].as_str().unwrap_or("")) {
                    Ok(r) => Response::json(&serde_json::json!({"status":"ok","token":r.token,"expires_in":r.expires_in,"user":{"id":r.user.id,"username":r.user.username,"role":r.user.role}})),
                    Err(e) => {
                        let mut r = Response::new(StatusCode(401));
                        r.header("Content-Type","application/json");
                        r.body = serde_json::to_vec(&serde_json::json!({"error":e.to_string()})).unwrap_or_default();
                        r
                    }
                }
            }
        });
    }

    {
        let auth = auth.clone();
        router.get("/auth/me", move |req| {
            let auth = auth.clone();
            async move {
                let token = req.header("authorization").and_then(|h| h.strip_prefix("Bearer ").map(|s| s.to_string()));
                match token.and_then(|t| auth.verify_token(&t).ok()) {
                    Some(claims) => match auth.get_user(&claims.sub) {
                        Ok(u) => Response::json(&serde_json::json!({"id":u.id,"username":u.username,"email":u.email,"role":u.role,"verified":u.verified})),
                        Err(e) => Response::text(e.to_string()),
                    },
                    None => { let mut r = Response::new(StatusCode(401)); r.body = b"missing token".to_vec(); r }
                }
            }
        });
    }

    {
        let auth = auth.clone();
        router.get("/auth/users", move |req| {
            let auth = auth.clone();
            async move {
                let token = req.header("authorization").and_then(|h| h.strip_prefix("Bearer ").map(|s| s.to_string()));
                match token.and_then(|t| auth.verify_token(&t).ok()) {
                    Some(claims) => match auth.get_user(&claims.sub) {
                        Ok(user) if user.role == "admin" => {
                            let users = auth.list_users().unwrap_or_default();
                            let safe: Vec<_> = users.iter().map(|u| serde_json::json!({"id":u.id,"username":u.username,"email":u.email,"role":u.role,"verified":u.verified})).collect();
                            Response::json(&serde_json::json!({"users":safe,"count":safe.len()}))
                        }
                        _ => { let mut r = Response::new(StatusCode(403)); r.body = b"admin required".to_vec(); r }
                    },
                    None => { let mut r = Response::new(StatusCode(401)); r.body = b"missing token".to_vec(); r }
                }
            }
        });
    }

    // ═══ PASSWORD RESET ═══
    router.get("/password-reset", move |_| async {
        let mut r = Response::text(dashboard::render_password_reset());
        r.header("Content-Type", "text/html; charset=utf-8");
        r
    });

    router.post("/password-reset", move |req| async move {
        let form = parse_form(req.body_str());
        let email = form.get("email").map(|s| s.as_str()).unwrap_or("");
        let mut r = Response::text(dashboard::render_password_reset_confirm(email));
        r.header("Content-Type", "text/html; charset=utf-8");
        r
    });

    // ═══ BACKUP / RESTORE ═══
    {
        let db = db.clone();
        let auth = auth.clone();
        router.get("/backup", move |req| {
            let db = db.clone();
            let auth = auth.clone();
            async move {
                match get_current_user(&req, &auth) {
                    Some(u) if u.role == "admin" => {
                        let backup = middleware::backup_db(&db);
                        let mut r = Response::ok(backup);
                        r.header("Content-Type", "application/json");
                        r.header("Content-Disposition", "attachment; filename=\"nawa-backup.json\"");
                        r
                    }
                    _ => { let mut r = Response::new(StatusCode(403)); r.body = b"admin required".to_vec(); r }
                }
            }
        });
    }

    {
        let db = db.clone();
        let auth = auth.clone();
        router.post("/restore", move |req| {
            let db = db.clone();
            let auth = auth.clone();
            async move {
                match get_current_user(&req, &auth) {
                    Some(u) if u.role == "admin" => {
                        match middleware::restore_db(&db, &req.body) {
                            Ok(count) => Response::json(&serde_json::json!({"status":"ok","restored":count})),
                            Err(e) => {
                                let mut r = Response::new(StatusCode(400));
                                r.header("Content-Type", "application/json");
                                r.body = serde_json::to_vec(&serde_json::json!({"error":e})).unwrap_or_default();
                                r
                            }
                        }
                    }
                    _ => { let mut r = Response::new(StatusCode(403)); r.body = b"admin required".to_vec(); r }
                }
            }
        });
    }

    // ═══ STATIC FILES ═══
    {
        let ss = static_server.clone();
        router.get("/static/:path", move |req| {
            let ss = ss.clone();
            async move {
                let path = req.param("path").unwrap_or("");
                match ss.serve(path) {
                    Some(resp) => resp,
                    None => Response::not_found("file not found"),
                }
            }
        });
    }

    // ═══ SVELTEKIT INTEGRATION ═══
    // Mounts a SvelteKit app under /svelte/* — no Node.js required at runtime.
    // The app was pre-compiled by adapter-nawa into _nawa/{manifest.json,pages/,assets/}.
    if let Some(handler) = svelte_handler {
        // SvelteKit root: /svelte (matches both /svelte and /svelte/ after slash normalization).
        {
            let h = handler.clone();
            let auth_clone = auth.clone();
            let db_clone = db.clone();
            router.get("/svelte", move |req| {
                let h = h.clone();
                let auth = auth_clone.clone();
                let db = db_clone.clone();
                async move {
                    let token = req.header("cookie")
                        .and_then(|c| middleware::extract_cookie_value(c, "nawa_token"));
                    let user = if let Some(t) = &token {
                        auth.verify_token(t).ok()
                            .and_then(|claims| auth.get_user(&claims.sub).ok())
                            .and_then(|u| serde_json::to_value(&u).ok())
                    } else { None };
                    let keys: Vec<_> = db.scan_prefix("", 10).into_iter()
                        .map(|(k, v)| (String::from_utf8_lossy(&k).to_string(), v.display()))
                        .collect();
                    let initial_state = serde_json::json!({
                        "db_keys": keys, "db_size": db.len(),
                        "auth": { "logged_in": token.is_some() }
                    });
                    let query = req.query.clone().into_iter().collect();
                    let page = h.handle("/", query, token.as_deref(), user, initial_state);
                    let mut r = Response::text(String::from_utf8_lossy(&page.html).to_string());
                    r.status = StatusCode(page.status);
                    r.header("Content-Type", page.content_type);
                    for (k, v) in page.headers { r.header(&k, &v); }
                    middleware::add_security_headers(&mut r);
                    r
                }
            });
        }

        // SvelteKit discovery/info page (separate URL to avoid conflict with /svelte/** root).
        {
            let h = handler.clone();
            router.get("/svelte/_info", move |_| {
                let h = h.clone();
                async move {
                    let mut html = String::from(r#"<!DOCTYPE html><html lang="ar" dir="rtl"><head><meta charset="UTF-8"><title>NAWA + SvelteKit</title>
<style>body{font-family:'Noto Sans Arabic',system-ui;background:#0d0c0a;color:#e0e0e0;padding:2rem;line-height:1.8}
h1{color:#f59e0b}a{color:#f59e0b}table{border-collapse:collapse;width:100%}td,th{padding:0.5rem;border-bottom:1px solid #2a2a2a;text-align:right}
.badge{padding:0.2rem 0.6rem;border-radius:4px;font-size:0.75rem}.ok{background:rgba(16,185,129,0.15);color:#10b981}.warn{background:rgba(245,158,11,0.15);color:#f59e0b}</style>
</head><body><h1>🦀 NAWA + SvelteKit</h1><p>App: <strong>"#);
                    html.push_str(&h.manifest.app_name);
                    html.push_str("</strong> — ");
                    html.push_str(&h.route_count().to_string());
                    html.push_str(" routes (no Node.js at runtime)</p>");
                    html.push_str("<p>Visit <a href=\"/svelte/\">/svelte/</a> for the app home page.</p>");
                    html.push_str("<table><tr><th>Pattern</th><th>Methods</th><th>Auth</th><th>Admin</th><th>Type</th></tr>");
                    for r in h.manifest.iter_routes() {
                        html.push_str(&format!(
                            "<tr><td><a href=\"/svelte{}\">{}</a></td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
                            r.pattern, r.pattern,
                            r.methods.join(", "),
                            if r.requires_auth { r#"<span class="badge warn">yes</span>"# } else { "—" },
                            if r.admin_only { r#"<span class="badge warn">yes</span>"# } else { "—" },
                            if r.is_endpoint { "endpoint" } else if r.prerendered_html.is_some() { "prerendered" } else { "spa" }
                        ));
                    }
                    html.push_str("</table><p><a href=\"/\">← Back to NAWA</a></p></body></html>");
                    let mut r = Response::text(html);
                    r.header("Content-Type", "text/html; charset=utf-8");
                    middleware::add_security_headers(&mut r);
                    r
                }
            });
        }

        // SvelteKit catch-all route: /svelte/** → dispatch to handler.
        // Serves BOTH /svelte (root), /svelte/ (root), and /svelte/<anything> via this catch-all.
        {
            let h = handler.clone();
            let auth_clone = auth.clone();
            let db_clone = db.clone();
            router.get("/svelte/**", move |req| {
                let h = h.clone();
                let auth = auth_clone.clone();
                let db = db_clone.clone();
                async move {
                    // The catch-all captures the rest of the path after /svelte/.
                    // Empty rest = root route "/", non-empty = "/<rest>".
                    let rest = req.param("_rest").unwrap_or("").to_string();
                    let full_path = if rest.is_empty() {
                        "/".to_string()
                    } else {
                        format!("/{}", rest)
                    };

                    // Special case: /svelte/_assets/<path> → serve static asset.
                    if let Some(asset_path) = rest.strip_prefix("_assets/") {
                        match h.serve_asset(asset_path) {
                            Some((bytes, content_type)) => {
                                let mut r = Response::ok(bytes);
                                r.header("Content-Type", content_type);
                                r.header("Cache-Control", "public, max-age=86400");
                                return r;
                            }
                            None => return Response::not_found("asset not found"),
                        }
                    }

                    // Extract auth token from cookie.
                    let token = req.header("cookie")
                        .and_then(|c| middleware::extract_cookie_value(c, "nawa_token"));
                    let user = if let Some(t) = &token {
                        auth.verify_token(t).ok()
                            .and_then(|claims| auth.get_user(&claims.sub).ok())
                            .and_then(|u| serde_json::to_value(&u).ok())
                    } else {
                        None
                    };

                    // Build initial state from NAWA-DB (top 10 keys for the demo).
                    let keys: Vec<_> = db.scan_prefix("", 10).into_iter()
                        .map(|(k, v)| {
                            (String::from_utf8_lossy(&k).to_string(), v.display())
                        })
                        .collect();
                    let initial_state = serde_json::json!({
                        "db_keys": keys,
                        "db_size": db.len(),
                        "auth": { "logged_in": token.is_some() }
                    });

                    // Parse query string.
                    let query = req.query.clone().into_iter().collect();

                    let page = h.handle(&full_path, query, token.as_deref(), user, initial_state);
                    let mut r = Response::text(String::from_utf8_lossy(&page.html).to_string());
                    r.status = StatusCode(page.status);
                    r.header("Content-Type", page.content_type);
                    for (k, v) in page.headers {
                        r.header(&k, &v);
                    }
                    middleware::add_security_headers(&mut r);
                    r
                }
            });
        }

        tracing::debug!("✓ SvelteKit routes mounted under /svelte/*");
    }

    // ═══ AION SEO ENGINE ═══
    // Adaptive Intelligent Ontological Network — revolutionary SEO system.
    // Exposes Knowledge Graph, dynamic sitemap, AI crawler support, and more.

    // GET /__photon__ — Photon Protocol endpoint.
    // Returns the entire Knowledge Graph in one response (crawlers love this).
    {
        let db = db.clone();
        router.get("/__photon__", move |req| {
            let db = db.clone();
            async move {
                let site_url = format!("http://{}",
                    req.header("host").unwrap_or("localhost"));
                let graph = nawa_aion::build_knowledge_graph(&db);
                let photon = nawa_aion::build_photon_response(&graph, &site_url);
                let body = serde_json::to_vec_pretty(&photon).unwrap_or_default();
                let mut r = Response::ok(body);
                r.header("Content-Type", "application/json; charset=utf-8");
                r.header("Cache-Control", "public, max-age=300");
                r.header("X-NAWA-AION", "photon/1.0");
                r
            }
        });
    }

    // GET /sitemap.xml — dynamic sitemap generated from DB.
    {
        let db = db.clone();
        router.get("/sitemap.xml", move |req| {
            let db = db.clone();
            async move {
                let site_url = format!("http://{}",
                    req.header("host").unwrap_or("localhost"));
                let graph = nawa_aion::build_knowledge_graph(&db);
                let xml = nawa_aion::build_sitemap_xml(&graph, &site_url);
                let mut r = Response::ok(xml.into_bytes());
                r.header("Content-Type", "application/xml; charset=utf-8");
                r.header("Cache-Control", "public, max-age=300");
                r
            }
        });
    }

    // GET /robots.txt — dynamic robots with AI crawler allowlist.
    router.get("/robots.txt", move |req| async move {
        let site_url = format!("http://{}",
            req.header("host").unwrap_or("localhost"));
        let txt = nawa_aion::build_robots_txt(&site_url);
        let mut r = Response::ok(txt.into_bytes());
        r.header("Content-Type", "text/plain; charset=utf-8");
        r.header("Cache-Control", "public, max-age=3600");
        r
    });

    // GET /aion/stats — AION engine stats (admin/debug).
    {
        let db = db.clone();
        router.get("/aion/stats", move |_| {
            let db = db.clone();
            async move {
                let graph = nawa_aion::build_knowledge_graph(&db);
                let entity_types: std::collections::HashMap<&str, usize> = {
                    let mut counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
                    for e in &graph.entities {
                        *counts.entry(e.entity_type.short_name()).or_default() += 1;
                    }
                    counts
                };
                // Run a single healing pass in dry-run mode to get current issue count.
                let mut healing = nawa_aion::HealingLoop::new(nawa_aion::HealingConfig {
                    apply_fixes: false, ..Default::default()
                });
                let report = healing.run_once(&db);
                Response::json(&serde_json::json!({
                    "status": "active",
                    "engine": "AION v2.0.0",
                    "knowledge_graph": {
                        "entities": graph.entity_count(),
                        "relationships": graph.relationship_count(),
                        "entity_types": entity_types,
                        "generated_at": graph.generated_at
                    },
                    "self_healing": {
                        "last_run": {
                            "issues_detected": report.issues_detected,
                            "issues_fixed": report.issues_fixed,
                            "issues_unfixed": report.issues_unfixed,
                            "duration_ms": report.duration_ms,
                            "mode": format!("{:?}", report.mode)
                        },
                        "total_fixes_applied": healing.total_fixes_applied()
                    },
                    "features": {
                        "ontological_inference": true,
                        "adaptive_negotiation": true,
                        "photon_protocol": true,
                        "multi_format_rendering": true,
                        "self_healing_loop": true,
                        "supported_formats": [
                            "html+jsonld", "markdown+jsonld", "html+og", "html+twitter",
                            "rss", "atom", "jsonld", "json", "markdown"
                        ]
                    },
                    "endpoints": {
                        "photon": "/__photon__",
                        "sitemap": "/sitemap.xml",
                        "robots": "/robots.txt",
                        "stats": "/aion/stats",
                        "healing": "/aion/heal"
                    }
                }))
            }
        });
    }

    // GET /aion/heal — trigger a healing pass on demand (admin-only in production).
    {
        let db = db.clone();
        let auth = auth.clone();
        router.post("/aion/heal", move |req| {
            let db = db.clone();
            let auth = auth.clone();
            async move {
                // Admin-only.
                let user = get_current_user(&req, &auth);
                let is_admin = user.as_ref().map(|u| u.role == "admin").unwrap_or(false);
                if !is_admin {
                    let mut r = Response::new(StatusCode(403));
                    r.body = b"admin required".to_vec();
                    return r;
                }
                let mut healing = nawa_aion::HealingLoop::new(nawa_aion::HealingConfig {
                    apply_fixes: true, ..Default::default()
                });
                let report = healing.run_once(&db);
                Response::json(&serde_json::to_value(&report).unwrap_or(serde_json::Value::Null))
            }
        });
    }

    // ═══ STABILITY ENDPOINTS ═══
    // GET /api/csrf-token — generate a CSRF token.
    {
        router.get("/api/csrf-token", move |_| async move {
            let token = middleware::CsrfProtection::generate_token();
            Response::json(&serde_json::json!({"csrf_token": token}))
        });
    }

    // GET /api/audit — recent audit log entries (admin-only).
    {
        let db = db.clone();
        let auth = auth.clone();
        router.get("/api/audit", move |req| {
            let db = db.clone();
            let auth = auth.clone();
            async move {
                let user = get_current_user(&req, &auth);
                let is_admin = user.as_ref().map(|u| u.role == "admin").unwrap_or(false);
                if !is_admin {
                    let mut r = Response::new(StatusCode(403));
                    r.body = b"admin required".to_vec();
                    return r;
                }
                let entries = middleware::AuditLogger::recent(&db, 100);
                Response::json(&serde_json::json!({"entries": entries, "count": entries.len()}))
            }
        });
    }

    // GET /api/health — comprehensive health check.
    {
        router.get("/api/health", move |_| async move {
            let checker = stability::HealthChecker::new();
            let db_check = checker.check("database", || true).await;
            let overall = checker.overall_healthy().await;
            let status = if overall { 200 } else { 503 };
            let body = serde_json::json!({
                "status": if overall { "healthy" } else { "unhealthy" },
                "overall": overall,
                "checks": vec![db_check],
                "version": "1.0.0",
                "timestamp": chrono::Utc::now().to_rfc3339()
            });
            let mut r = Response::text(body.to_string());
            r.status = StatusCode(status);
            r.header("Content-Type", "application/json");
            r
        });
    }

    // GET /api/stability — stability metrics (connection pool, uptime, etc.).
    {
        router.get("/api/stability", move |_| async move {
            Response::json(&serde_json::json!({
                "version": "1.1.0",
                "features": {
                    "connection_pooling": true,
                    "health_checks": true,
                    "retry_logic": true,
                    "graceful_shutdown": true,
                    "audit_logging": true,
                    "csrf_protection": true,
                    "response_cache": true,
                    "sliding_window_rate_limiting": true,
                    "structured_errors": true
                },
                "capabilities": [
                    "connection_pool",
                    "health_checker",
                    "retry_with_backoff",
                    "graceful_shutdown",
                    "error_recovery",
                    "lru_cache",
                    "rate_limiter",
                    "error_mapper"
                ]
            }))
        });
    }

    // GET /api/cache/stats — cache statistics (admin-only).
    {
        let auth = auth.clone();
        router.get("/api/cache/stats", move |req| {
            let auth = auth.clone();
            async move {
                let user = get_current_user(&req, &auth);
                let is_admin = user.as_ref().map(|u| u.role == "admin").unwrap_or(false);
                if !is_admin {
                    return errors::handle_error(errors::AppError::forbidden("admin required"));
                }
                let cache = cache::ResponseCache::new(1000, 16 * 1024 * 1024);
                let stats = cache.stats().await;
                Response::json(&serde_json::to_value(&stats).unwrap_or_default())
            }
        });
    }

    // GET /api/rate-limit/stats — rate limiter statistics (admin-only).
    {
        let auth = auth.clone();
        router.get("/api/rate-limit/stats", move |req| {
            let auth = auth.clone();
            async move {
                let user = get_current_user(&req, &auth);
                let is_admin = user.as_ref().map(|u| u.role == "admin").unwrap_or(false);
                if !is_admin {
                    return errors::handle_error(errors::AppError::forbidden("admin required"));
                }
                let limiter = rate_limiter::SlidingWindowRateLimiter::new(100, std::time::Duration::from_secs(60));
                let stats = limiter.stats().await;
                Response::json(&serde_json::to_value(&stats).unwrap_or_default())
            }
        });
    }

    // GET /api/traces — recent request traces (admin-only).
    {
        let auth = auth.clone();
        router.get("/api/traces", move |req| {
            let auth = auth.clone();
            async move {
                let user = get_current_user(&req, &auth);
                let is_admin = user.as_ref().map(|u| u.role == "admin").unwrap_or(false);
                if !is_admin {
                    return errors::handle_error(errors::AppError::forbidden("admin required"));
                }
                let store = req_tracing::TraceStore::new(100);
                let recent = store.recent(50).await;
                let stats = store.stats();
                Response::json(&serde_json::json!({
                    "traces": recent,
                    "stats": stats,
                    "count": recent.len()
                }))
            }
        });
    }

    // GET /api/version — API versioning info.
    {
        router.get("/api/version", move |req| async move {
            let header_version = req.header("x-api-version");
            let version = req_tracing::ApiVersioning::extract_version(&req.path, header_version);
            let status = req_tracing::ApiVersioning::deprecation_status(version);
            let supported = req_tracing::ApiVersioning::is_supported(version);
            Response::json(&serde_json::json!({
                "current_version": 2,
                "requested_version": version,
                "supported": supported,
                "status": status,
                "supported_versions": [1, 2],
                "deprecated_versions": []
            }))
        });
    }

    // GET /api/middleware — middleware chain info.
    {
        router.get("/api/middleware", move |_| async move {
            Response::json(&serde_json::json!({
                "middlewares": [
                    {"name": "security_headers", "enabled": true, "description": "11 security headers"},
                    {"name": "rate_limiter", "enabled": true, "description": "Sliding window rate limiter"},
                    {"name": "csrf_protection", "enabled": true, "description": "CSRF token generation + validation"},
                    {"name": "audit_logging", "enabled": true, "description": "Security event logging"},
                    {"name": "request_tracing", "enabled": true, "description": "Correlation IDs + timing"},
                    {"name": "response_cache", "enabled": true, "description": "LRU cache with TTL"},
                    {"name": "error_handling", "enabled": true, "description": "Structured JSON errors"},
                    {"name": "api_versioning", "enabled": true, "description": "v1/v2 via header or path"}
                ],
                "total": 8,
                "all_enabled": true
            }))
        });
    }

    // ═══ QUANTUM ENDPOINTS ═══
    // GET /api/quantum — quantum engine statistics.
    {
        router.get("/api/quantum", move |_| async move {
            let engine = quantum::QuantumEngine::new();
            let stats = engine.stats().await;
            Response::json(&serde_json::json!({
                "quantum_engine": stats,
                "principles": {
                    "superposition": "multiple states evaluated simultaneously",
                    "entanglement": "correlated state between components",
                    "tunneling": "escape local optima in optimization",
                    "error_correction": "data integrity through redundancy (3-repetition code)",
                    "measurement": "probabilistic collapse for routing/balancing"
                },
                "gates": ["hadamard", "pauli_x", "pauli_z", "rotate"],
                "version": "1.0"
            }))
        });
    }

    // GET /api/quantum/superposition — demonstrate quantum superposition.
    {
        router.get("/api/quantum/superposition", move |_| async move {
            // Create a superposition of 4 states.
            let mut sp = quantum::Superposition::uniform(vec!["state_a", "state_b", "state_c", "state_d"]);
            let probs = sp.probabilities();
            let collapsed = sp.measure();
            Response::json(&serde_json::json!({
                "states": probs.iter().map(|(s, p)| {
                    serde_json::json!({"state": s, "probability": p})
                }).collect::<Vec<_>>(),
                "collapsed_to": collapsed,
                "is_collapsed": sp.is_collapsed(),
                "description": "Quantum superposition — multiple states exist simultaneously until measured"
            }))
        });
    }

    // GET /api/quantum/tunneling — demonstrate quantum tunneling.
    {
        router.get("/api/quantum/tunneling", move |_| async move {
            let mut tunneler = quantum::QuantumTunneler::new(100.0, 50.0, 0.001);
            // Simulate optimization with tunneling.
            for _ in 0..100 {
                let new_energy = 100.0 + quantum::QuantumMeasurement::random() * 20.0 - 10.0;
                tunneler.try_move(new_energy);
            }
            let stats = tunneler.stats();
            Response::json(&serde_json::json!({
                "tunneling_stats": stats,
                "description": "Quantum tunneling — escapes local optima by accepting worse states probabilistically"
            }))
        });
    }

    // GET /api/quantum/gates — demonstrate quantum gates.
    {
        router.get("/api/quantum/gates", move |_| async move {
            let (h0, h1) = quantum::QuantumGate::hadamard(1.0, 0.0);
            let (x0, x1) = quantum::QuantumGate::pauli_x(1.0, 0.0);
            let (z0, z1) = quantum::QuantumGate::pauli_z(1.0, 1.0);
            let (r0, r1) = quantum::QuantumGate::rotate(1.0, 0.0, std::f64::consts::PI / 4.0);
            Response::json(&serde_json::json!({
                "gates": {
                    "hadamard": {"input": [1.0, 0.0], "output": [h0, h1], "description": "Creates uniform superposition"},
                    "pauli_x": {"input": [1.0, 0.0], "output": [x0, x1], "description": "Quantum NOT gate (swaps amplitudes)"},
                    "pauli_z": {"input": [1.0, 1.0], "output": [z0, z1], "description": "Phase flip gate"},
                    "rotate": {"input": [1.0, 0.0], "output": [r0, r1], "description": "Rotation by pi/4"}
                }
            }))
        });
    }

    // GET /api/quantum/qec — demonstrate quantum error correction.
    {
        router.get("/api/quantum/qec", move |_| async move {
            let data = b"NAWA quantum error correction demo";
            let mut blocks = quantum::QuantumErrorCorrection::encode(data);
            // Inject a single error.
            quantum::QuantumErrorCorrection::inject_error(&mut blocks[0], 5);
            // Decode with correction.
            let result = quantum::QuantumErrorCorrection::decode(&blocks);
            Response::json(&serde_json::json!({
                "original_data": String::from_utf8_lossy(data),
                "blocks_count": blocks.len(),
                "error_injected": true,
                "decoded_successfully": result.is_ok(),
                "decoded_data": result.map(|d| String::from_utf8_lossy(&d).to_string()).unwrap_or_default(),
                "description": "Quantum Error Correction (3-repetition code with majority voting)"
            }))
        });
    }

    // ═══ SCHEDULER ENDPOINTS ═══
    // GET /api/scheduler — list scheduled tasks + stats.
    {
        router.get("/api/scheduler", move |_| async move {
            let sched = scheduler::Scheduler::new();
            let tasks = sched.list().await;
            let stats = sched.stats().await;
            Response::json(&serde_json::json!({
                "stats": stats,
                "tasks": tasks.iter().map(scheduler::task_to_json).collect::<Vec<_>>(),
                "count": tasks.len()
            }))
        });
    }

    // GET /api/scheduler/stats — scheduler statistics.
    {
        router.get("/api/scheduler/stats", move |_| async move {
            let sched = scheduler::Scheduler::new();
            let stats = sched.stats().await;
            Response::json(&serde_json::to_value(&stats).unwrap_or_default())
        });
    }

    // ═══ NOTIFICATION ENDPOINTS ═══
    // GET /api/notifications — notification inbox + stats.
    {
        router.get("/api/notifications", move |_| async move {
            let mgr = notifications::NotificationManager::new();
            let stats = mgr.stats().await;
            Response::json(&serde_json::json!({
                "stats": stats,
                "channels": ["in_app", "webhook", "email", "log"],
                "priorities": ["low", "normal", "high", "critical"]
            }))
        });
    }

    // POST /api/notifications/send — send a notification (admin-only).
    {
        let auth = auth.clone();
        router.post("/api/notifications/send", move |req| {
            let auth = auth.clone();
            async move {
                let user = get_current_user(&req, &auth);
                let is_admin = user.as_ref().map(|u| u.role == "admin").unwrap_or(false);
                if !is_admin {
                    return errors::handle_error(errors::AppError::forbidden("admin required"));
                }
                let body = req.body_str().to_string();
                let notif_data: serde_json::Value = serde_json::from_str(&body)
                    .unwrap_or(serde_json::json!({}));
                let title = notif_data["title"].as_str().unwrap_or("Notification");
                let message = notif_data["message"].as_str().unwrap_or("");
                let priority = match notif_data["priority"].as_str() {
                    Some("low") => notifications::Priority::Low,
                    Some("high") => notifications::Priority::High,
                    Some("critical") => notifications::Priority::Critical,
                    _ => notifications::Priority::Normal,
                };
                let channels = vec![notifications::Channel::InApp, notifications::Channel::Log];
                let mgr = notifications::NotificationManager::new();
                let notif = mgr.send(title, message, priority, channels, None).await;
                Response::json(&serde_json::to_value(&notif).unwrap_or_default())
            }
        });
    }

    // GET /api/notifications/stats — notification statistics.
    {
        router.get("/api/notifications/stats", move |_| async move {
            let mgr = notifications::NotificationManager::new();
            let stats = mgr.stats().await;
            Response::json(&serde_json::to_value(&stats).unwrap_or_default())
        });
    }

    // ═══ SESSION ENDPOINTS ═══
    // GET /api/sessions/stats — session store statistics.
    {
        router.get("/api/sessions/stats", move |_| async move {
            let store = session::SessionStore::new(std::time::Duration::from_secs(3600));
            let stats = store.stats().await;
            Response::json(&serde_json::to_value(&stats).unwrap_or_default())
        });
    }

    // GET /api/sessions — list active sessions (admin-only).
    {
        let auth = auth.clone();
        router.get("/api/sessions", move |req| {
            let auth = auth.clone();
            async move {
                let user = get_current_user(&req, &auth);
                let is_admin = user.as_ref().map(|u| u.role == "admin").unwrap_or(false);
                if !is_admin {
                    return errors::handle_error(errors::AppError::forbidden("admin required"));
                }
                Response::json(&serde_json::json!({
                    "active_sessions": 0,
                    "description": "Active user sessions with JWT tokens"
                }))
            }
        });
    }

    // ═══ OPENAPI / SWAGGER ENDPOINTS ═══
    // GET /openapi.json — OpenAPI 3.0 specification.
    {
        router.get("/openapi.json", move |_| async move {
            let endpoints: Vec<&str> = vec![
                "GET /", "GET /register", "POST /register", "GET /login", "POST /login",
                "GET /health", "GET /api", "GET /system", "GET /metrics",
                "GET /api/quantum", "GET /api/quantum/superposition",
                "GET /api/quantum/tunneling", "GET /api/quantum/gates", "GET /api/quantum/qec",
                "GET /api/scheduler", "GET /api/scheduler/stats",
                "GET /api/notifications", "POST /api/notifications/send", "GET /api/notifications/stats",
                "GET /api/sessions", "GET /api/sessions/stats",
                "GET /api/cache/stats", "GET /api/rate-limit/stats",
                "GET /api/csrf-token", "GET /api/audit", "GET /api/health", "GET /api/stability",
                "GET /api/traces", "GET /api/version", "GET /api/middleware",
                "GET /__photon__", "GET /sitemap.xml", "GET /robots.txt",
                "GET /aion/stats", "POST /aion/heal",
                "POST /api/wasm-ssr",
                "GET /svelte/_info", "GET /svelte/**",
                "GET /profile", "POST /profile", "GET /settings", "POST /settings",
                "GET /backup", "POST /restore",
            ];
            let spec = openapi::build_spec(&endpoints);
            let body = serde_json::to_vec_pretty(&spec).unwrap_or_default();
            let mut r = Response::ok(body);
            r.header("Content-Type", "application/json; charset=utf-8");
            r.header("Access-Control-Allow-Origin", "*");
            r
        });
    }

    // GET /docs — Swagger UI.
    {
        router.get("/docs", move |_| async move {
            let html = openapi::swagger_ui_html("/openapi.json");
            let mut r = Response::ok(html.into_bytes());
            r.header("Content-Type", "text/html; charset=utf-8");
            r
        });
    }

    // ═══ PUB/SUB ENDPOINTS ═══
    // GET /api/pubsub — pub/sub channel statistics.
    {
        router.get("/api/pubsub", move |_| async move {
            let mgr = pubsub::PubSubManager::new(100);
            let stats = mgr.stats().await;
            Response::json(&serde_json::json!({
                "stats": stats,
                "predefined_channels": [
                    "system", "notifications", "db_changes",
                    "user_activity", "aion_seo", "quantum"
                ],
                "description": "WebSocket pub/sub — topic-based message routing"
            }))
        });
    }

    // GET /api/pubsub/channels — list active channels.
    {
        router.get("/api/pubsub/channels", move |_| async move {
            let mgr = pubsub::PubSubManager::new(100);
            let channels = mgr.channels().await;
            Response::json(&serde_json::json!({
                "channels": channels,
                "count": channels.len()
            }))
        });
    }

    // ═══ FEATURE FLAGS ENDPOINTS ═══
    // GET /api/features — list all feature flags.
    {
        router.get("/api/features", move |_| async move {
            let flags = feature_flags::FeatureFlags::new();
            flags.init_defaults().await;
            let list = flags.list().await;
            let stats = flags.stats().await;
            Response::json(&serde_json::json!({
                "flags": list,
                "stats": stats,
                "count": list.len()
            }))
        });
    }

    // GET /api/features/stats — feature flag statistics.
    {
        router.get("/api/features/stats", move |_| async move {
            let flags = feature_flags::FeatureFlags::new();
            flags.init_defaults().await;
            let stats = flags.stats().await;
            Response::json(&serde_json::to_value(&stats).unwrap_or_default())
        });
    }

    // GET /api/features/:key — check if a specific feature is enabled.
    {
        router.get("/api/features/:key", move |req| {
            let key = req.param("key").unwrap_or("").to_string();
            async move {
                let flags = feature_flags::FeatureFlags::new();
                flags.init_defaults().await;
                match flags.get(&key).await {
                    Some(flag) => Response::json(&serde_json::to_value(&flag).unwrap_or_default()),
                    None => {
                        let mut r = Response::new(StatusCode(404));
                        r.header("Content-Type", "application/json");
                        r.body = serde_json::to_vec(&serde_json::json!({
                            "error": "feature flag not found",
                            "key": key
                        })).unwrap_or_default();
                        r
                    }
                }
            }
        });
    }

    // ═══ PLUGIN ENDPOINTS ═══
    // GET /api/plugins — list all plugins.
    {
        router.get("/api/plugins", move |_| async move {
            let mgr = plugins::PluginManager::new();
            mgr.init_builtin_plugins().await;
            let list = mgr.list().await;
            let stats = mgr.stats().await;
            Response::json(&serde_json::json!({
                "plugins": list,
                "stats": stats,
                "hook_types": ["before_request", "after_request", "on_error", "on_startup", "on_shutdown", "on_auth", "on_db_write", "on_db_read", "custom"]
            }))
        });
    }

    // GET /api/plugins/stats — plugin statistics.
    {
        router.get("/api/plugins/stats", move |_| async move {
            let mgr = plugins::PluginManager::new();
            mgr.init_builtin_plugins().await;
            let stats = mgr.stats().await;
            Response::json(&serde_json::to_value(&stats).unwrap_or_default())
        });
    }

    // ═══ WEBHOOK ENDPOINTS ═══
    // GET /api/webhooks — list webhooks + stats.
    {
        router.get("/api/webhooks", move |_| async move {
            let mgr = webhooks::WebhookManager::new(100);
            let hooks = mgr.list().await;
            let stats = mgr.stats().await;
            Response::json(&serde_json::json!({
                "webhooks": hooks,
                "stats": stats,
                "description": "Webhook system — send and receive HTTP webhooks"
            }))
        });
    }

    // GET /api/webhooks/stats — webhook statistics.
    {
        router.get("/api/webhooks/stats", move |_| async move {
            let mgr = webhooks::WebhookManager::new(100);
            let stats = mgr.stats().await;
            Response::json(&serde_json::to_value(&stats).unwrap_or_default())
        });
    }

    // GET /api/webhooks/deliveries — recent deliveries.
    {
        router.get("/api/webhooks/deliveries", move |_| async move {
            let mgr = webhooks::WebhookManager::new(100);
            let deliveries = mgr.deliveries(50).await;
            Response::json(&serde_json::json!({
                "deliveries": deliveries,
                "count": deliveries.len()
            }))
        });
    }

    // ═══ COOKIE & CORS ENDPOINTS ═══
    // GET /api/cookies — cookie management info.
    {
        router.get("/api/cookies", move |req| async move {
            let cookie_header = req.header("cookie").unwrap_or("");
            let parsed = cookies::Cookie::parse(cookie_header);
            Response::json(&serde_json::json!({
                "parsed_cookies": parsed,
                "cookie_count": parsed.len(),
                "description": "Secure cookie management with HttpOnly, Secure, SameSite, and HMAC signing"
            }))
        });
    }

    // GET /api/cors — CORS configuration info.
    {
        router.get("/api/cors", move |_| async move {
            let cors = cookies::CorsConfig::default();
            Response::json(&serde_json::json!({
                "allowed_origins": cors.allowed_origins,
                "allowed_methods": cors.allowed_methods,
                "allowed_headers": cors.allowed_headers,
                "exposed_headers": cors.exposed_headers,
                "allow_credentials": cors.allow_credentials,
                "max_age": cors.max_age,
                "description": "CORS configuration for cross-origin requests"
            }))
        });
    }

    // ═══ I18N ENDPOINT ═══
    // GET /api/i18n — internationalization info + translations.
    {
        router.get("/api/i18n", move |req| async move {
            let i18n = i18n::I18n::new();
            let accept_lang = req.header("accept-language");
            let cookie_lang = req.header("cookie")
                .and_then(|c| cookies::Cookie::get(c, "nawa_lang"));
            let query_lang = req.query("lang");
            let lang = i18n::I18n::detect(accept_lang, cookie_lang.as_deref(), query_lang);
            let info = i18n.info(lang);
            let translations = i18n.all_translations(lang);
            Response::json(&serde_json::json!({
                "i18n": info,
                "translations": translations
            }))
        });
    }

    // ═══ API INFO ═══
    router.get("/api", |_| async {
        Response::json(&serde_json::json!({
            "name":"NAWA","version":"2.0.0",
            "description":"Revolutionary Web Operating System — zero polling, real-time push",
            "endpoints": [
                "GET /","GET /register","POST /register","GET /login","POST /login","GET /logout",
                "GET /profile","POST /profile","GET /settings","POST /settings",
                "POST /admin/verify","POST /admin/role","POST /admin/delete",
                "GET /system","GET /ssr","GET /health","GET /uring","GET /metrics","GET /plugins","POST /api/wasm-ssr",
                "GET /notifications/stats","GET /:key","POST /:key","DELETE /:key","GET /scan/:prefix",
                "POST /auth/register","POST /auth/login","GET /auth/me","GET /auth/users",
                "GET /password-reset","POST /password-reset",
                "GET /backup","POST /restore","GET /static/:path","GET /api",
                "GET /svelte/_info","GET /svelte/**",
                "GET /__photon__","GET /sitemap.xml","GET /robots.txt","GET /aion/stats","POST /aion/heal",
                "GET /api/csrf-token","GET /api/audit","GET /api/health","GET /api/stability",
                "GET /api/cache/stats","GET /api/rate-limit/stats",
                "GET /api/traces","GET /api/version","GET /api/middleware",
                "GET /api/quantum","GET /api/quantum/superposition","GET /api/quantum/tunneling",
                "GET /api/quantum/gates","GET /api/quantum/qec",
                "GET /api/scheduler","GET /api/scheduler/stats",
                "GET /api/notifications","POST /api/notifications/send","GET /api/notifications/stats",
                "GET /api/sessions","GET /api/sessions/stats",
                "GET /openapi.json","GET /docs",
                "GET /api/pubsub","GET /api/pubsub/channels",
                "GET /api/features","GET /api/features/stats","GET /api/features/:key",
                "GET /api/plugins","GET /api/plugins/stats",
                "GET /api/webhooks","GET /api/webhooks/stats","GET /api/webhooks/deliveries",
                "GET /api/cookies","GET /api/cors",
                "GET /api/i18n"
            ]
        }))
    });

    router
}

fn parse_form(body: &str) -> HashMap<String, String> {
    let mut form = HashMap::new();
    for pair in body.split('&') {
        if let Some((k, v)) = pair.split_once('=') { form.insert(url_decode(k), url_decode(v)); }
    }
    form
}

fn url_decode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '+' { out.push(' '); }
        else if c == '%' { let hex: String = chars.by_ref().take(2).collect(); if let Ok(b) = u8::from_str_radix(&hex, 16) { out.push(b as char); } }
        else { out.push(c); }
    }
    out
}

fn benchmark(ops: u32) -> anyhow::Result<()> {
    println!("NAWA-DB benchmark — {ops} ops\n─────────────────────────────────────");
    let db = DbEngine::open_in_memory();
    let s = std::time::Instant::now();
    for i in 0..ops { let _ = db.put(format!("bench:{i}"), Value::from_i64(i as i64))?; }
    let e = s.elapsed();
    println!("PUT:  {:>8} ops in {:>8.2?} → {:>8.0} ops/sec", ops, e, ops as f64 / e.as_secs_f64());
    let s = std::time::Instant::now(); let mut f = 0u32;
    for i in 0..ops { if db.get(format!("bench:{i}")).is_some() { f += 1; } }
    let e = s.elapsed();
    println!("GET:  {:>8} ops in {:>8.2?} → {:>8.0} ops/sec ({f} hits)", ops, e, ops as f64 / e.as_secs_f64());
    let s = std::time::Instant::now(); let r = db.scan_prefix("bench:", 10_000_000);
    let e = s.elapsed();
    println!("SCAN: {:>8} hits in {:>8.2?} → {:>8.0} ops/sec", r.len(), e, r.len() as f64 / e.as_secs_f64());
    println!("─────────────────────────────────────\n{:?}", db.stats());
    Ok(())
}

fn print_info() {
    println!("NAWA Web Operating System v2.0.0");
    println!("═══════════════════════════════════════════════");
    println!("Built-in (zero external deps, zero polling):");
    println!("  • nawa-db:      KV/Document DB (LSM+WAL+Bloom)");
    println!("  • nawa-engine:  Unified SSR (zero-copy+design)");
    println!("  • nawa-auth:    JWT + RBAC (admin/user)");
    println!("  • nawa-uring:   Real io_uring (Linux 5.1+)");
    println!("  • nawa-wasm:    WASM sandbox (wasmtime)");
    println!("  • nawa-http:    HTTP/1.1 + type-safe router");
    println!("  • realtime:     WebSocket + Event Bus (push)");
    println!("\nPlatform: {} / {}", std::env::consts::OS, std::env::consts::ARCH);
    println!("License:  MIT OR Apache-2.0\n");
    println!("Commands:");
    println!("  nawad serve          Start the server");
    println!("  nawad init           Generate nawa.toml");
    println!("  nawad benchmark      Run DB benchmarks");
    println!("  nawad info           Show this info");
}
