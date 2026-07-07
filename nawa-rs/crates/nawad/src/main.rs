//! # nawad — NAWA Web Operating System
//!
//! محرك ويب ثوري في binary واحد. لا يحتاج أي شيء خارجي.
//! لا polling — كل شيء event-driven (WebSocket push).

mod config;
mod dashboard;
mod metrics;
mod middleware;
mod realtime;

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
        Commands::Serve { addr, data_dir, plugins_dir, static_dir, svelte_dir, config: cfg_path, no_wal_sync } => {
            let mut cfg = config::Config::load(&cfg_path);
            if let Some(a) = addr { cfg.addr = a; }
            if let Some(d) = data_dir { cfg.data_dir = d.display().to_string(); }
            if let Some(p) = plugins_dir { cfg.plugins_dir = p.display().to_string(); }
            if let Some(s) = static_dir { cfg.static_dir = s.display().to_string(); }
            if no_wal_sync { cfg.wal_sync = false; }
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(serve(cfg, svelte_dir))
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

async fn serve(cfg: config::Config, svelte_dir: Option<PathBuf>) -> anyhow::Result<()> {
    tracing::info!("╔══════════════════════════════════════════════╗");
    tracing::info!("║  NAWA Web Operating System v0.1.0            ║");
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
                    "engine": "AION v0.1.0-alpha",
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

    // ═══ API INFO ═══
    router.get("/api", |_| async {
        Response::json(&serde_json::json!({
            "name":"NAWA","version":"0.1.0-alpha",
            "description":"Revolutionary Web Operating System — zero polling, real-time push",
            "endpoints": [
                "GET /","GET /register","POST /register","GET /login","POST /login","GET /logout",
                "GET /profile","POST /profile","GET /settings","POST /settings",
                "POST /admin/verify","POST /admin/role","POST /admin/delete",
                "GET /system","GET /ssr","GET /health","GET /uring","GET /metrics","GET /plugins",
                "GET /notifications/stats","GET /:key","POST /:key","DELETE /:key","GET /scan/:prefix",
                "POST /auth/register","POST /auth/login","GET /auth/me","GET /auth/users",
                "GET /password-reset","POST /password-reset",
                "GET /backup","POST /restore","GET /static/:path","GET /api",
                "GET /svelte/_info","GET /svelte/**",
                "GET /__photon__","GET /sitemap.xml","GET /robots.txt","GET /aion/stats","POST /aion/heal"
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
    println!("NAWA Web Operating System v0.1.0-alpha");
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
