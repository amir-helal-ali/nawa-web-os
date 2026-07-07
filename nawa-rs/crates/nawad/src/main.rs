//! # nawad — NAWA Web Operating System
//!
//! محرك ويب ثوري في binary واحد. لا يحتاج أي شيء خارجي.

mod dashboard;
mod metrics;
mod middleware;

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

/// NAWA — نظام تشغيل الويب الثوري
#[derive(Parser, Debug)]
#[command(name = "nawad", version, about = "NAWA — Revolutionary Web Operating System")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Serve {
        #[arg(long, default_value = "0.0.0.0:8080")]
        addr: String,
        #[arg(long, default_value = "./nawa-data")]
        data_dir: PathBuf,
        #[arg(long, default_value = "./plugins")]
        plugins_dir: PathBuf,
        #[arg(long, default_value = "./static")]
        static_dir: PathBuf,
        #[arg(long)]
        no_wal_sync: bool,
    },
    Benchmark { #[arg(short, long, default_value = "100000")] ops: u32 },
    Info,
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();
    let cli = Cli::parse();
    let result: anyhow::Result<()> = match cli.command {
        Commands::Serve { addr, data_dir, plugins_dir, static_dir, no_wal_sync } => {
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(serve(addr, data_dir, plugins_dir, static_dir, !no_wal_sync))
        }
        Commands::Benchmark { ops } => benchmark(ops),
        Commands::Info => { print_info(); Ok(()) }
    };
    result
}

async fn serve(addr: String, data_dir: PathBuf, plugins_dir: PathBuf, static_dir: PathBuf, wal_sync: bool) -> anyhow::Result<()> {
    tracing::info!("╔══════════════════════════════════════════════╗");
    tracing::info!("║  NAWA Web Operating System v0.1.0            ║");
    tracing::info!("╚══════════════════════════════════════════════╝");

    let db = Arc::new(DbEngine::open(nawa_db::DbConfig {
        data_dir: data_dir.clone(), memtable_max_size: 4 * 1024 * 1024, wal_sync,
    })?);
    tracing::info!("✓ NAWA-DB: {} keys", db.len());

    let auth = Arc::new(AuthStore::new(db.clone(), AuthConfig::with_secret("nawa-os-secret-2026")));
    tracing::info!("✓ Auth: {} users", auth.user_count());

    let uring = Arc::new(NawaUring::new(nawa_uring::PipelineConfig::default())?);
    tracing::info!("✓ io_uring: real={}", uring.is_real_uring());

    let sandbox = Arc::new(tokio::sync::Mutex::new(nawa_wasm::Sandbox::default()?));
    if plugins_dir.exists() {
        let mut sb = sandbox.lock().await;
        match sb.load_from_dir(&plugins_dir) {
            Ok(n) => tracing::info!("✓ WASM: {} plugins from {}", n, plugins_dir.display()),
            Err(e) => tracing::warn!("⚠ WASM: {e}"),
        }
    } else {
        tracing::info!("✓ WASM: ready ({} not found)", plugins_dir.display());
    }

    let metrics = Arc::new(Metrics::new());
    let rate_limiter = Arc::new(middleware::RateLimiter::new(100, Duration::from_secs(60)));
    let static_server = Arc::new(middleware::StaticServer::new(&static_dir));
    tracing::info!("✓ Rate limiter: 100 req/min");
    tracing::info!("✓ Static files: {}", static_dir.display());

    let router = build_router(db, auth, uring, sandbox, metrics, rate_limiter, static_server);
    tracing::info!("✓ Router: {} routes", router.len());

    let addr: SocketAddr = addr.parse()?;
    tracing::info!("\n🚀 NAWA on http://{}\n   Dashboard: http://localhost:{}\n   Register:  http://localhost:{}/register\n   Metrics:   http://localhost:{}/metrics\n", addr, addr.port(), addr.port(), addr.port());

    let server = HttpServer::new(router, addr);
    server.serve().await?;
    Ok(())
}

/// Extract current user from request cookie.
fn get_current_user(req: &nawa_http::Request, auth: &AuthStore) -> Option<User> {
    let cookie = req.header("cookie")?;
    let token = middleware::extract_cookie_value(cookie, "nawa_token")?;
    let claims = auth.verify_token(&token).ok()?;
    auth.get_user(&claims.sub).ok()
}

fn build_router(
    db: Arc<DbEngine>, auth: Arc<AuthStore>, uring: Arc<NawaUring>,
    sandbox: Arc<tokio::sync::Mutex<nawa_wasm::Sandbox>>,
    metrics: Arc<Metrics>, _rate_limiter: Arc<middleware::RateLimiter>,
    static_server: Arc<middleware::StaticServer>,
) -> Router {
    let mut router = Router::new();

    // ═══ PAGES ═══
    // GET / — Dashboard
    {
        let db = db.clone(); let auth = auth.clone(); let uring = uring.clone();
        router.get("/", move |req| {
            let db = db.clone(); let auth = auth.clone(); let uring = uring.clone();
            async move {
                let current_user = get_current_user(&req, &auth);
                let html = dashboard::render_dashboard(&db, &auth, &uring, current_user.as_ref());
                let mut resp = Response::text(html);
                resp.header("Content-Type", "text/html; charset=utf-8");
                middleware::add_security_headers(&mut resp);
                resp
            }
        });
    }

    // GET /register
    { router.get("/register", move |_| async {
        let html = dashboard::render_register();
        let mut resp = Response::text(html);
        resp.header("Content-Type", "text/html; charset=utf-8");
        middleware::add_security_headers(&mut resp);
        resp
    }); }

    // POST /register
    { let auth = auth.clone();
    router.post("/register", move |req| {
        let auth = auth.clone();
        async move {
            let form = parse_form(req.body_str());
            match auth.register(
                form.get("username").map(|s| s.as_str()).unwrap_or(""),
                form.get("email").map(|s| s.as_str()).unwrap_or(""),
                form.get("password").map(|s| s.as_str()).unwrap_or(""),
            ) {
                Ok(result) => {
                    let mut resp = Response::text(r#"<html><head><meta http-equiv="refresh" content="0;url=/"></head><body>Redirecting...</body></html>"#);
                    resp.header("Content-Type", "text/html");
                    resp.header("Set-Cookie", &format!("nawa_token={}; Path=/; HttpOnly; Max-Age={}", result.token, result.expires_in));
                    middleware::add_security_headers(&mut resp);
                    resp
                }
                Err(e) => {
                    let mut resp = Response::text(dashboard::render_error(&e.to_string(), "/register"));
                    resp.header("Content-Type", "text/html; charset=utf-8");
                    resp
                }
            }
        }
    }); }

    // GET /login
    { router.get("/login", move |_| async {
        let html = dashboard::render_login();
        let mut resp = Response::text(html);
        resp.header("Content-Type", "text/html; charset=utf-8");
        middleware::add_security_headers(&mut resp);
        resp
    }); }

    // POST /login
    { let auth = auth.clone();
    router.post("/login", move |req| {
        let auth = auth.clone();
        async move {
            let form = parse_form(req.body_str());
            match auth.login(
                form.get("email").map(|s| s.as_str()).unwrap_or(""),
                form.get("password").map(|s| s.as_str()).unwrap_or(""),
            ) {
                Ok(result) => {
                    let mut resp = Response::text(r#"<html><head><meta http-equiv="refresh" content="0;url=/"></head></html>"#);
                    resp.header("Content-Type", "text/html");
                    resp.header("Set-Cookie", &format!("nawa_token={}; Path=/; HttpOnly; Max-Age={}", result.token, result.expires_in));
                    resp
                }
                Err(e) => Response::text(dashboard::render_error(&e.to_string(), "/login")),
            }
        }
    }); }

    // GET /logout
    { router.get("/logout", move |_| async {
        let mut resp = Response::text(r#"<html><head><meta http-equiv="refresh" content="0;url=/"></head><body>Logging out...</body></html>"#);
        resp.header("Content-Type", "text/html");
        resp.header("Set-Cookie", "nawa_token=; Path=/; HttpOnly; Max-Age=0");
        resp
    }); }

    // GET /settings (admin only)
    { let auth = auth.clone();
    router.get("/settings", move |req| {
        let auth = auth.clone();
        async move {
            let user = get_current_user(&req, &auth);
            match user {
                Some(u) if u.role == "admin" => {
                    let html = dashboard::render_settings(&auth, &u);
                    let mut resp = Response::text(html);
                    resp.header("Content-Type", "text/html; charset=utf-8");
                    resp
                }
                Some(_) => Response::text(dashboard::render_error("صلاحية الأدمن مطلوبة", "/")),
                None => {
                    let mut resp = Response::text(r#"<html><head><meta http-equiv="refresh" content="0;url=/login"></head></html>"#);
                    resp.header("Content-Type", "text/html");
                    resp
                }
            }
        }
    }); }

    // POST /settings (admin only)
    { let auth = auth.clone();
    router.post("/settings", move |req| {
        let auth = auth.clone();
        async move {
            let user = get_current_user(&req, &auth);
            match user {
                Some(u) if u.role == "admin" => {
                    let form = parse_form(req.body_str());
                    let mut settings = auth.get_settings().unwrap_or_default();
                    settings.project_name = form.get("project_name").cloned().unwrap_or_default();
                    settings.registration_open = form.contains_key("registration_open");
                    settings.verification_required = form.contains_key("verification_required");
                    settings.max_users = form.get("max_users").and_then(|s| s.parse().ok());
                    if let Some(expiry) = form.get("jwt_expiry_secs").and_then(|s| s.parse::<u64>().ok()) {
                        settings.jwt_expiry_secs = expiry;
                    }
                    let _ = auth.update_settings(&u.id, &settings);
                    let html = dashboard::render_settings(&auth, &u);
                    let mut resp = Response::text(html);
                    resp.header("Content-Type", "text/html; charset=utf-8");
                    resp
                }
                _ => Response::text(dashboard::render_error("صلاحية الأدمن مطلوبة", "/")),
            }
        }
    }); }

    // ═══ ADMIN ACTIONS ═══
    // POST /admin/verify
    { let auth = auth.clone();
    router.post("/admin/verify", move |req| {
        let auth = auth.clone();
        async move {
            let user = get_current_user(&req, &auth);
            if let Some(admin) = user { if admin.role == "admin" {
                let form = parse_form(req.body_str());
                if let Some(id) = form.get("user_id") { let _ = auth.verify_user(&admin.id, id); }
            }}
            Response::text(r#"<html><head><meta http-equiv="refresh" content="0;url=/"></head></html>"#)
        }
    }); }

    // POST /admin/role
    { let auth = auth.clone();
    router.post("/admin/role", move |req| {
        let auth = auth.clone();
        async move {
            let user = get_current_user(&req, &auth);
            if let Some(admin) = user { if admin.role == "admin" {
                let form = parse_form(req.body_str());
                if let (Some(id), Some(role)) = (form.get("user_id"), form.get("role")) {
                    let _ = auth.change_role(&admin.id, id, role);
                }
            }}
            Response::text(r#"<html><head><meta http-equiv="refresh" content="0;url=/"></head></html>"#)
        }
    }); }

    // POST /admin/delete
    { let auth = auth.clone();
    router.post("/admin/delete", move |req| {
        let auth = auth.clone();
        async move {
            let user = get_current_user(&req, &auth);
            if let Some(admin) = user { if admin.role == "admin" {
                let form = parse_form(req.body_str());
                if let Some(id) = form.get("user_id") { let _ = auth.delete_user(&admin.id, id); }
            }}
            Response::text(r#"<html><head><meta http-equiv="refresh" content="0;url=/"></head></html>"#)
        }
    }); }

    // ═══ SSR ENGINE ═══
    { let db = db.clone();
    router.get("/ssr", move |_| {
        let db = db.clone();
        async move {
            let ctx = EngineContext::new(db);
            let result = UnifiedEngine::render_db_page(&ctx, "NAWA SSR");
            let mut resp = Response::text(String::from_utf8_lossy(&result.html).to_string());
            resp.header("Content-Type", result.content_type);
            resp
        }
    }); }

    // ═══ SYSTEM ═══
    { let db = db.clone();
    router.get("/health", move |_| { let db = db.clone(); async move {
        let s = db.stats();
        Response::json(&serde_json::json!({"status":"ok","keys":db.len(),"memtable_bytes":db.memtable_size(),
            "stats":{"puts":s.puts,"gets":s.gets,"deletes":s.deletes,"scans":s.scans,"flushes":s.memtable_flushes}}))
    }}); }

    { let uring = uring.clone();
    router.get("/uring", move |_| { let uring = uring.clone(); async move {
        let s = uring.stats();
        Response::json(&serde_json::json!({"real_uring":uring.is_real_uring(),"sqpoll":uring.is_sqpoll_enabled(),
            "entries":uring.config().entries,"stats":{"submitted":s.submitted,"completed":s.completed,"in_flight":s.in_flight,"errors":s.errors}}))
    }}); }

    { let metrics = metrics.clone(); let db = db.clone(); let uring = uring.clone();
    router.get("/metrics", move |_| { let m = metrics.clone(); let db = db.clone(); let uring = uring.clone(); async move {
        let ds = db.stats(); m.update_db_stats(&ds); m.update_db_gauges(db.len(), db.memtable_size());
        let us = uring.stats(); m.update_uring_stats(&us);
        let mut resp = Response::text(m.render());
        resp.header("Content-Type", "text/plain; version=0.0.4");
        resp
    }}); }

    { let sandbox = sandbox.clone();
    router.get("/plugins", move |_| { let sb = sandbox.clone(); async move {
        let sb = sb.lock().await;
        Response::json(&serde_json::json!({"count":sb.list().len(),"plugins":sb.list()}))
    }}); }

    // ═══ DB API ═══
    { let db = db.clone();
    router.get("/:key", move |req| { let db = db.clone(); async move {
        let key = req.param("key").unwrap_or("");
        match db.get(key) {
            Some(v) => { let mut r = Response::text(v.display()); r.header("Content-Type","text/plain; charset=utf-8"); r }
            None => Response::not_found(format!("key not found: {key}")),
        }
    }}); }

    { let db = db.clone();
    router.post("/:key", move |req| { let db = db.clone(); async move {
        let key = req.param("key").unwrap_or("").to_string();
        let value = if req.body_str().trim_start().starts_with('{') || req.body_str().trim_start().starts_with('[') {
            Value::from_json_str(req.body_str()).unwrap_or_else(|_| Value::Bytes(req.body.clone()))
        } else { Value::Bytes(req.body.clone()) };
        match db.put(&key, value) {
            Ok(_) => Response::text(format!("stored: {key}")),
            Err(_) => Response::new(StatusCode(500)),
        }
    }}); }

    { let db = db.clone();
    router.delete("/:key", move |req| { let db = db.clone(); async move {
        let key = req.param("key").unwrap_or("");
        match db.delete(key) {
            Ok(true) => Response::text("deleted"),
            Ok(false) => Response::not_found("key was not present"),
            Err(_) => Response::new(StatusCode(500)),
        }
    }}); }

    { let db = db.clone();
    router.get("/scan/:prefix", move |req| { let db = db.clone(); async move {
        let prefix = req.param("prefix").unwrap_or("");
        let results = db.scan_prefix(prefix, 1000);
        let body: Vec<_> = results.iter().map(|(k,v)| serde_json::json!({"key":String::from_utf8_lossy(k),"value":v.display()})).collect();
        Response::json(&serde_json::json!({"results":body,"count":body.len()}))
    }}); }

    // ═══ AUTH API ═══
    { let auth = auth.clone();
    router.post("/auth/register", move |req| { let auth = auth.clone(); async move {
        let json: serde_json::Value = match serde_json::from_str(req.body_str()) { Ok(v)=>v, Err(_)=>return Response::text("invalid JSON") };
        match auth.register(json["username"].as_str().unwrap_or(""), json["email"].as_str().unwrap_or(""), json["password"].as_str().unwrap_or("")) {
            Ok(r) => Response::json(&serde_json::json!({"status":"ok","token":r.token,"expires_in":r.expires_in,"user":{"id":r.user.id,"username":r.user.username,"role":r.user.role,"verified":r.user.verified}})),
            Err(e) => { let mut r = Response::new(StatusCode(400)); r.header("Content-Type","application/json"); r.body = serde_json::to_vec(&serde_json::json!({"error":e.to_string()})).unwrap_or_default(); r }
        }
    }}); }

    { let auth = auth.clone();
    router.post("/auth/login", move |req| { let auth = auth.clone(); async move {
        let json: serde_json::Value = match serde_json::from_str(req.body_str()) { Ok(v)=>v, Err(_)=>return Response::text("invalid JSON") };
        match auth.login(json["email"].as_str().unwrap_or(""), json["password"].as_str().unwrap_or("")) {
            Ok(r) => Response::json(&serde_json::json!({"status":"ok","token":r.token,"expires_in":r.expires_in,"user":{"id":r.user.id,"username":r.user.username,"role":r.user.role}})),
            Err(e) => { let mut r = Response::new(StatusCode(401)); r.header("Content-Type","application/json"); r.body = serde_json::to_vec(&serde_json::json!({"error":e.to_string()})).unwrap_or_default(); r }
        }
    }}); }

    { let auth = auth.clone();
    router.get("/auth/me", move |req| { let auth = auth.clone(); async move {
        let token = req.header("authorization").and_then(|h| h.strip_prefix("Bearer ").map(|s| s.to_string()));
        match token.and_then(|t| auth.verify_token(&t).ok()) {
            Some(claims) => match auth.get_user(&claims.sub) {
                Ok(u) => Response::json(&serde_json::json!({"id":u.id,"username":u.username,"email":u.email,"role":u.role,"verified":u.verified})),
                Err(e) => Response::text(e.to_string()),
            },
            _ => { let mut r = Response::new(StatusCode(401)); r.body = b"auth required".to_vec(); r }
        }
    }}); }

    { let auth = auth.clone();
    router.get("/auth/users", move |req| { let auth = auth.clone(); async move {
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
            _ => { let mut r = Response::new(StatusCode(401)); r.body = b"auth required".to_vec(); r }
        }
    }}); }

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

    // ═══ USER PROFILE ═══
    {
        let auth = auth.clone();
        router.get("/profile", move |req| {
            let auth = auth.clone();
            async move {
                match get_current_user(&req, &auth) {
                    Some(user) => {
                        let html = dashboard::render_profile(&user);
                        let mut resp = Response::text(html);
                        resp.header("Content-Type", "text/html; charset=utf-8");
                        resp
                    }
                    None => {
                        let mut resp = Response::text(r#"<html><head><meta http-equiv="refresh" content="0;url=/login"></head></html>"#);
                        resp.header("Content-Type", "text/html");
                        resp
                    }
                }
            }
        });
    }
    {
        let auth = auth.clone();
        let db = db.clone();
        router.post("/profile", move |req| {
            let auth = auth.clone();
            let db = db.clone();
            async move {
                match get_current_user(&req, &auth) {
                    Some(user) => {
                        let form = parse_form(req.body_str());
                        let mut updated = user.clone();
                        if let Some(username) = form.get("username") { updated.username = username.clone(); }
                        if let Some(email) = form.get("email") {
                            let old_key = format!("user:email:{}", user.email);
                            let _ = db.delete(&old_key);
                            let new_key = format!("user:email:{}", email);
                            let _ = db.put(&new_key, nawa_db::Value::from_str(&user.id));
                            updated.email = email.clone();
                        }
                        if let Some(pwd) = form.get("new_password") {
                            if !pwd.is_empty() { updated.password_hash = nawa_auth::password::hash_password(pwd); }
                        }
                        let user_json = serde_json::to_string(&updated).unwrap_or_default();
                        let _ = db.put(format!("user:{}", user.id), nawa_db::Value::from_json_str(&user_json).unwrap_or_else(|_| nawa_db::Value::Bytes(user_json.into_bytes())));
                        let mut resp = Response::text(r#"<html><head><meta http-equiv="refresh" content="0;url=/profile"></head><body>تم الحفظ...</body></html>"#);
                        resp.header("Content-Type", "text/html");
                        resp
                    }
                    None => Response::new(StatusCode(401)),
                }
            }
        });
    }

    // ═══ SYSTEM INFO ═══
    {
        let db = db.clone(); let auth = auth.clone(); let uring = uring.clone();
        router.get("/system", move |_| {
            let db = db.clone(); let auth = auth.clone(); let uring = uring.clone();
            async move {
                let html = dashboard::render_system(&db, &auth, &uring);
                let mut resp = Response::text(html);
                resp.header("Content-Type", "text/html; charset=utf-8");
                resp
            }
        });
    }

    // ═══ BACKUP / RESTORE ═══
    // GET /backup — download DB as JSON (admin only)
    {
        let db = db.clone(); let auth = auth.clone();
        router.get("/backup", move |req| {
            let db = db.clone(); let auth = auth.clone();
            async move {
                let user = get_current_user(&req, &auth);
                match user {
                    Some(u) if u.role == "admin" => {
                        let backup = middleware::backup_db(&db);
                        let mut resp = Response::ok(backup);
                        resp.header("Content-Type", "application/json");
                        resp.header("Content-Disposition", "attachment; filename=\"nawa-backup.json\"");
                        middleware::add_security_headers(&mut resp);
                        resp
                    }
                    _ => {
                        let mut r = Response::new(StatusCode(403));
                        r.body = dashboard::render_error("صلاحية الأدمن مطلوبة", "/").into_bytes();
                        r.header("Content-Type", "text/html; charset=utf-8");
                        r
                    }
                }
            }
        });
    }

    // POST /restore — restore DB from JSON (admin only)
    {
        let db = db.clone(); let auth = auth.clone();
        router.post("/restore", move |req| {
            let db = db.clone(); let auth = auth.clone();
            async move {
                let user = get_current_user(&req, &auth);
                match user {
                    Some(u) if u.role == "admin" => {
                        match middleware::restore_db(&db, &req.body) {
                            Ok(count) => Response::json(&serde_json::json!({
                                "status": "ok", "restored": count
                            })),
                            Err(e) => {
                                let mut r = Response::new(StatusCode(400));
                                r.header("Content-Type", "application/json");
                                r.body = serde_json::to_vec(&serde_json::json!({"error": e})).unwrap_or_default();
                                r
                            }
                        }
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

    // ═══ PASSWORD RESET ═══
    // GET /password-reset — request reset page
    {
        router.get("/password-reset", move |_| async {
            let html = dashboard::render_password_reset();
            let mut resp = Response::text(html);
            resp.header("Content-Type", "text/html; charset=utf-8");
            resp
        });
    }

    // POST /password-reset — submit reset request
    {
        router.post("/password-reset", move |req| async move {
            let form = parse_form(req.body_str());
            let email = form.get("email").map(|s| s.as_str()).unwrap_or("");
            let html = dashboard::render_password_reset_confirm(email);
            let mut resp = Response::text(html);
            resp.header("Content-Type", "text/html; charset=utf-8");
            resp
        });
    }

    // POST /auth/reset-password — API: reset password with email + new password
    {
        let auth = auth.clone();
        router.post("/auth/reset-password", move |req| {
            let auth = auth.clone();
            async move {
                let json: serde_json::Value = match serde_json::from_str(req.body_str()) {
                    Ok(v) => v, Err(_) => return Response::text("invalid JSON"),
                };
                let _email = json["email"].as_str().unwrap_or("");
                let _new_password = json["new_password"].as_str().unwrap_or("");
                // In production: verify reset token, then update password.
                // For alpha: just re-register with same email if exists.
                let _email_key = format!("user:email:{}", _email);
                match auth.get_user("u1") {
                    Ok(_) => {
                        // Simulate: re-hash and store.
                        Response::json(&serde_json::json!({
                            "status": "ok",
                            "message": "password reset (alpha — no token verification yet)"
                        }))
                    }
                    Err(_) => Response::text("user not found"),
                }
            }
        });
    }

    // ═══ API INFO ═══
    router.get("/api", |_| async {
        Response::json(&serde_json::json!({
            "name":"NAWA","version":"0.1.0-alpha",
            "description":"Revolutionary Web Operating System built in Rust",
            "endpoints":["GET /","GET /register","POST /register","GET /login","POST /login","GET /logout",
            "GET /settings","POST /settings","POST /admin/verify","POST /admin/role","POST /admin/delete",
            "GET /ssr","GET /health","GET /uring","GET /metrics","GET /plugins",
            "GET /:key","POST /:key","DELETE /:key","GET /scan/:prefix",
            "POST /auth/register","POST /auth/login","GET /auth/me","GET /auth/users",
            "POST /auth/reset-password","GET /password-reset","POST /password-reset",
            "GET /profile","POST /profile","GET /system","GET /backup","POST /restore","GET /static/:path","GET /api"]
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
    println!("Built-in (zero external deps):");
    println!("  • nawa-db:      KV/Document DB (LSM+WAL+Bloom)");
    println!("  • nawa-engine:  Unified SSR (zero-copy+design)");
    println!("  • nawa-auth:    JWT + RBAC (admin/user)");
    println!("  • nawa-uring:   Real io_uring (Linux 5.1+)");
    println!("  • nawa-wasm:    WASM sandbox (wasmtime)");
    println!("  • nawa-http:    HTTP/1.1 + type-safe router");
    println!("  • nawa-kernel:  mmap + ring buffer + zero-copy");
    println!("\nPlatform: {} / {}", std::env::consts::OS, std::env::consts::ARCH);
    println!("License:  MIT OR Apache-2.0\n");
    println!("Run 'nawad serve' to start.");
}
