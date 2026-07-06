//! # nawad — NAWA daemon
//!
//! The single binary that runs the entire NAWA system:
//! HTTP server + NAWA-DB + zero-copy kernel.

mod metrics;

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use clap::{Parser, Subcommand};
use metrics::Metrics;
use nawa_db::{DbEngine, Value};
use nawa_auth::{AuthConfig, AuthStore};
use nawa_engine::{UnifiedEngine, EngineContext};
use nawa_http::{HttpServer, Response, Router, StatusCode};
use tracing_subscriber::EnvFilter;

/// NAWA daemon — the revolutionary web operating system.
#[derive(Parser, Debug)]
#[command(name = "nawad", version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Start the HTTP server.
    Serve {
        /// Address to bind on.
        #[arg(long, default_value = "0.0.0.0:8080")]
        addr: String,
        /// Data directory for NAWA-DB.
        #[arg(long, default_value = "./nawa-data")]
        data_dir: PathBuf,
        /// Disable WAL sync (faster but less durable).
        #[arg(long)]
        no_wal_sync: bool,
    },
    /// Run a benchmark.
    Benchmark {
        /// Number of operations.
        #[arg(short, long, default_value = "10000")]
        ops: u32,
    },
    /// Print version info.
    Info,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Serve {
            addr,
            data_dir,
            no_wal_sync,
        } => serve(addr, data_dir, !no_wal_sync).await,
        Commands::Benchmark { ops } => benchmark(ops),
        Commands::Info => {
            print_info();
            Ok(())
        }
    }
}

async fn serve(addr: String, data_dir: PathBuf, wal_sync: bool) -> anyhow::Result<()> {
    tracing::info!("NAWA daemon v0.1.0-alpha starting");
    tracing::info!("data_dir: {}", data_dir.display());

    let db_config = nawa_db::DbConfig {
        data_dir: data_dir.clone(),
        memtable_max_size: 4 * 1024 * 1024,
        wal_sync,
    };
    let db = Arc::new(DbEngine::open(db_config)?);
    tracing::info!("NAWA-DB opened — {} keys", db.len());

    let sandbox = Arc::new(tokio::sync::Mutex::new(
        nawa_wasm::Sandbox::default().map_err(|e| anyhow::anyhow!("sandbox init failed: {e}"))?,
    ));
    tracing::info!("WASM sandbox initialized — {} plugins", sandbox.lock().await.len());

    // Initialize auth system.
    let auth = Arc::new(AuthStore::new(db.clone(), AuthConfig::with_secret("nawad-secret-key")));
    tracing::info!("Auth system initialized — {} users", auth.user_count());

    // Initialize io_uring pipeline.
    let uring_config = nawa_uring::PipelineConfig::default();
    let uring = Arc::new(nawa_uring::NawaUring::new(uring_config)?);
    tracing::info!(
        "io_uring pipeline initialized — real_uring={}, sqpoll={}, entries={}",
        uring.is_real_uring(),
        uring.is_sqpoll_enabled(),
        uring.config().entries
    );

    // Initialize Prometheus metrics.
    let metrics = Arc::new(Metrics::new());
    tracing::info!("Prometheus metrics initialized — /metrics endpoint");

    let mut router = Router::new();

    // Health check
    {
        let db = db.clone();
        router.get("/health", move |_| {
            let db = db.clone();
            async move {
                let stats = db.stats();
                let body = serde_json::json!({
                    "status": "ok",
                    "keys": db.len(),
                    "memtable_bytes": db.memtable_size(),
                    "stats": {
                        "puts": stats.puts,
                        "gets": stats.gets,
                        "deletes": stats.deletes,
                        "scans": stats.scans,
                        "flushes": stats.memtable_flushes,
                    }
                });
                Response::json(&body)
            }
        });
    }

    // GET /:key — fetch a value
    {
        let db = db.clone();
        router.get("/:key", move |req| {
            let db = db.clone();
            async move {
                let key = req.param("key").unwrap_or("");
                match db.get(key) {
                    Some(v) => Response::text(v.display()),
                    None => Response::not_found(format!("key not found: {key}")),
                }
            }
        });
    }

    // POST /:key — store a value (body as raw bytes)
    {
        let db = db.clone();
        router.post("/:key", move |req| {
            let db = db.clone();
            async move {
                let key = req.param("key").unwrap_or("").to_string();
                // Try to parse as JSON; fall back to bytes.
                let value = if req.body_str().trim_start().starts_with('{')
                    || req.body_str().trim_start().starts_with('[')
                {
                    Value::from_json_str(req.body_str()).unwrap_or_else(|_| {
                        Value::Bytes(req.body.clone())
                    })
                } else {
                    Value::Bytes(req.body.clone())
                };
                match db.put(&key, value) {
                    Ok(_seq) => Response::text(format!("stored: {key}")),
                    Err(_e) => {
                        let mut r = Response::new(StatusCode::INTERNAL_SERVER_ERROR);
                        r.header("Content-Type", "text/plain");
                        r.body = b"internal server error".to_vec();
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
                    Err(_) => Response::new(StatusCode::INTERNAL_SERVER_ERROR),
                }
            }
        });
    }

    // GET /scan/:prefix — list keys with prefix
    {
        let db = db.clone();
        router.get("/scan/:prefix", move |req| {
            let db = db.clone();
            async move {
                let prefix = req.param("prefix").unwrap_or("");
                let results = db.scan_prefix(prefix, 1000);
                let body: Vec<serde_json::Value> = results
                    .iter()
                    .map(|(k, v)| {
                        serde_json::json!({
                            "key": String::from_utf8_lossy(k),
                            "value": v.display(),
                        })
                    })
                    .collect();
                Response::json(&serde_json::json!({ "results": body, "count": body.len() }))
            }
        });
    }

    // GET /uring — io_uring pipeline stats
    {
        let uring = uring.clone();
        router.get("/uring", move |_| {
            let uring = uring.clone();
            async move {
                let stats = uring.stats();
                let body = serde_json::json!({
                    "real_uring": uring.is_real_uring(),
                    "sqpoll_enabled": uring.is_sqpoll_enabled(),
                    "entries": uring.config().entries,
                    "stats": {
                        "submitted": stats.submitted,
                        "completed": stats.completed,
                        "in_flight": stats.in_flight,
                        "bytes_transferred": stats.bytes_transferred,
                        "errors": stats.errors,
                    }
                });
                Response::json(&body)
            }
        });
    }

    // GET /metrics — Prometheus metrics endpoint
    {
        let metrics = metrics.clone();
        let db = db.clone();
        let uring = uring.clone();
        router.get("/metrics", move |_| {
            let metrics = metrics.clone();
            let db = db.clone();
            let uring = uring.clone();
            async move {
                // Update gauges from current state.
                let db_stats = db.stats();
                metrics.update_db_stats(&db_stats);
                metrics.update_db_gauges(db.len(), db.memtable_size());

                let uring_stats = uring.stats();
                metrics.update_uring_stats(&uring_stats);

                // Render in Prometheus text format.
                let body = metrics.render();
                let mut resp = Response::text(body);
                resp.header("Content-Type", "text/plain; version=0.0.4; charset=utf-8");
                resp
            }
        });
    }

    // GET /plugins — list loaded WASM plugins
    {
        let sandbox = sandbox.clone();
        router.get("/plugins", move |_| {
            let sandbox = sandbox.clone();
            async move {
                let sb = sandbox.lock().await;
                let plugins = sb.list();
                let body = serde_json::json!({
                    "count": plugins.len(),
                    "plugins": plugins,
                });
                Response::json(&body)
            }
        });
    }

    // POST /plugins/:name/invoke — invoke a plugin function
    {
        let sandbox = sandbox.clone();
        router.post("/plugins/:name/invoke", move |req| {
            let sandbox = sandbox.clone();
            async move {
                let name = req.param("name").unwrap_or("").to_string();
                let func = req.body_str().to_string();
                let sb = sandbox.lock().await;
                match sb.invoke(&name, &func) {
                    Ok(result) => {
                        let body = serde_json::json!({
                            "plugin": name,
                            "function": func,
                            "result": result,
                            "status": "ok"
                        });
                        Response::json(&body)
                    }
                    Err(e) => {
                        let body = serde_json::json!({
                            "plugin": name,
                            "function": func,
                            "error": e.to_string(),
                            "status": "error"
                        });
                        let mut r = Response::new(StatusCode(500));
                        r.header("Content-Type", "application/json");
                        r.body = serde_json::to_vec(&body).unwrap_or_default();
                        r
                    }
                }
            }
        });
    }

    // GET /unified — Unified Engine: DB → ZeroCopyHtml → response (true zero-copy)
    {
        let db = db.clone();
        router.get("/unified", move |_| {
            let db = db.clone();
            async move {
                let ctx = EngineContext::new(db);
                let result = UnifiedEngine::render_db_page(&ctx, "NAWA Unified Engine");
                let mut resp = Response::text(String::from_utf8_lossy(&result.html).to_string());
                resp.header("Content-Type", result.content_type);
                resp
            }
        });
    }

    // ─── Auth Endpoints ───
    // POST /auth/register
    {
        let auth = auth.clone();
        router.post("/auth/register", move |req| {
            let auth = auth.clone();
            async move {
                let body = req.body_str();
                let json: serde_json::Value = match serde_json::from_str(body) {
                    Ok(v) => v,
                    Err(_) => return Response::text("invalid JSON"),
                };
                let username = json["username"].as_str().unwrap_or("");
                let email = json["email"].as_str().unwrap_or("");
                let password = json["password"].as_str().unwrap_or("");
                match auth.register(username, email, password) {
                    Ok(result) => Response::json(&serde_json::json!({
                        "status": "ok",
                        "user": {
                            "id": result.user.id,
                            "username": result.user.username,
                            "role": result.user.role,
                            "verified": result.user.verified
                        },
                        "token": result.token,
                        "expires_in": result.expires_in
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

    // POST /auth/login
    {
        let auth = auth.clone();
        router.post("/auth/login", move |req| {
            let auth = auth.clone();
            async move {
                let body = req.body_str();
                let json: serde_json::Value = match serde_json::from_str(body) {
                    Ok(v) => v,
                    Err(_) => return Response::text("invalid JSON"),
                };
                let email = json["email"].as_str().unwrap_or("");
                let password = json["password"].as_str().unwrap_or("");
                match auth.login(email, password) {
                    Ok(result) => Response::json(&serde_json::json!({
                        "status": "ok",
                        "user": {
                            "id": result.user.id,
                            "username": result.user.username,
                            "role": result.user.role
                        },
                        "token": result.token,
                        "expires_in": result.expires_in
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

    // GET /auth/me (requires token in Authorization header)
    {
        let auth = auth.clone();
        router.get("/auth/me", move |req| {
            let auth = auth.clone();
            async move {
                let token = req.header("authorization")
                    .and_then(|h| h.strip_prefix("Bearer ").map(|s| s.to_string()))
                    .or_else(|| req.header("authorization").map(|s| s.to_string()));
                match token {
                    Some(t) => match auth.verify_token(&t) {
                        Ok(claims) => match auth.get_user(&claims.sub) {
                            Ok(user) => Response::json(&serde_json::json!({
                                "id": user.id,
                                "username": user.username,
                                "email": user.email,
                                "role": user.role,
                                "verified": user.verified
                            })),
                            Err(e) => Response::text(e.to_string()),
                        },
                        Err(e) => {
                            let mut r = Response::new(StatusCode(401));
                            r.body = e.to_string().into_bytes();
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

    // GET /auth/users (admin only)
    {
        let auth = auth.clone();
        router.get("/auth/users", move |req| {
            let auth = auth.clone();
            async move {
                let token = req.header("authorization")
                    .and_then(|h| h.strip_prefix("Bearer ").map(|s| s.to_string()))
                    .or_else(|| req.header("authorization").map(|s| s.to_string()));
                match token.and_then(|t| auth.verify_token(&t).ok()) {
                    Some(claims) => match auth.get_user(&claims.sub) {
                        Ok(user) if user.role == "admin" => {
                            let users = auth.list_users().unwrap_or_default();
                            let safe: Vec<_> = users.iter().map(|u| serde_json::json!({
                                "id": u.id, "username": u.username,
                                "email": u.email, "role": u.role, "verified": u.verified
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

    // GET /ssr — Unified Engine SSR (uses nawa-engine, the unified pipeline)
    {
        let db = db.clone();
        router.get("/ssr", move |_| {
            let db = db.clone();
            async move {
                let ctx = EngineContext::new(db);
                let result = UnifiedEngine::render_db_page(&ctx, "NAWA SSR");
                let mut resp = Response::text(String::from_utf8_lossy(&result.html).to_string());
                resp.header("Content-Type", result.content_type);
                resp
            }
        });
    }

    // GET / — root info
    router.get("/", |_| async {
        let body = serde_json::json!({
            "name": "NAWA",
            "version": "0.1.0-alpha",
            "description": "Revolutionary Web Operating System built in Rust",
            "endpoints": [
                "GET /health",
                "GET /uring",
                "GET /metrics",
                "GET /plugins",
                "GET /:key",
                "POST /:key",
                "DELETE /:key",
                "GET /scan/:prefix",
                "POST /plugins/:name/invoke",
            ]
        });
        Response::json(&body)
    });

    let addr: SocketAddr = addr.parse()?;
    let server = HttpServer::new(router, addr);
    server.serve().await?;
    Ok(())
}

fn benchmark(ops: u32) -> anyhow::Result<()> {
    use std::time::Instant;
    println!("NAWA-DB benchmark — {ops} operations");
    println!("─────────────────────────────────────");

    let db = DbEngine::open_in_memory();

    // PUT benchmark
    let started = Instant::now();
    for i in 0..ops {
        let key = format!("bench:{i}");
        let _ = db.put(&key, Value::from_i64(i as i64))?;
    }
    let put_elapsed = started.elapsed();
    println!(
        "PUT:  {:>8} ops in {:>8.2?}  →  {:>8.0} ops/sec",
        ops,
        put_elapsed,
        ops as f64 / put_elapsed.as_secs_f64()
    );

    // GET benchmark
    let started = Instant::now();
    let mut found = 0u32;
    for i in 0..ops {
        let key = format!("bench:{i}");
        if db.get(&key).is_some() {
            found += 1;
        }
    }
    let get_elapsed = started.elapsed();
    println!(
        "GET:  {:>8} ops in {:>8.2?}  →  {:>8.0} ops/sec  ({found} hits)",
        ops,
        get_elapsed,
        ops as f64 / get_elapsed.as_secs_f64()
    );

    // SCAN benchmark
    let started = Instant::now();
    let results = db.scan_prefix("bench:", 10_000_000);
    let scan_elapsed = started.elapsed();
    println!(
        "SCAN: {:>8} hits in {:>8.2?}  →  {:>8.0} ops/sec",
        results.len(),
        scan_elapsed,
        results.len() as f64 / scan_elapsed.as_secs_f64()
    );

    println!("─────────────────────────────────────");
    println!("Stats: {:?}", db.stats());
    Ok(())
}

fn print_info() {
    println!("nawad v0.1.0-alpha (NAWA Web Operating System)");
    println!("─────────────────────────────────────────────");
    println!("Built with: Rust 1.83+");
    println!("Components:");
    println!("  • nawa-kernel: io_uring + mmap + zero-copy");
    println!("  • nawa-db:     MemTable + SSTable + WAL + Bloom filter");
    println!("  • nawa-http:   HTTP/1.1 server + type-safe router");
    println!();
    println!("License: MIT OR Apache-2.0");
    println!("Repo:    https://github.com/amir-helal-ali/nawa-web-os");
    println!();
    println!("Run 'nawad serve' to start the HTTP server.");
    println!("Run 'nawad benchmark --ops 100000' to benchmark DB.");
}
