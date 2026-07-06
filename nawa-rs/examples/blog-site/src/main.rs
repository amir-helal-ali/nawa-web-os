//! # Blog Site — Example NAWA Website
//!
//! A complete blog application built with NAWA-DB.
//! Features: Create/list/get/delete posts, HTML pages, JSON API, stats.

use nawa_db::{DbEngine, Value};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("╔══════════════════════════════════════════════╗");
    println!("║  NAWA Blog Site v0.1.0                       ║");
    println!("║  مثال موقع مدونة بُني بـ NAWA                  ║");
    println!("╚══════════════════════════════════════════════╝\n");

    let db = Arc::new(DbEngine::open_in_memory());
    println!("✓ NAWA-DB initialized");

    seed_sample_posts(&db)?;
    println!("✓ Sample data loaded — {} posts", db.len());

    let addr: SocketAddr = "0.0.0.0:8080".parse()?;
    println!("\n🚀 Blog site running!");
    println!("   Home:   http://localhost:8080/");
    println!("   API:    http://localhost:8080/api/posts");
    println!("   Health: http://localhost:8080/health\n");
    println!("Press Ctrl+C to stop\n");

    let listener = TcpListener::bind(addr).await?;
    loop {
        let (stream, _) = listener.accept().await?;
        let db = db.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, db).await {
                eprintln!("connection error: {e}");
            }
        });
    }
}

async fn handle_connection(
    stream: TcpStream,
    db: Arc<DbEngine>,
) -> anyhow::Result<()> {
    let (mut reader, mut writer) = stream.into_split();
    let mut buf = Vec::new();
    let mut tmp = [0u8; 1024];

    // Read until headers complete
    loop {
        let n = reader.read(&mut tmp).await?;
        if n == 0 {
            return Ok(());
        }
        buf.extend_from_slice(&tmp[..n]);
        if buf.windows(4).any(|w| w == b"\r\n\r\n") {
            break;
        }
        if buf.len() > 64 * 1024 {
            return Ok(());
        }
    }

    let header_end = buf.windows(4).position(|w| w == b"\r\n\r\n").unwrap();
    let header_str = String::from_utf8_lossy(&buf[..header_end]);
    let mut lines = header_str.split("\r\n");
    let request_line = lines.next().unwrap_or("");
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("GET");
    let path = parts.next().unwrap_or("/");

    // Parse body if POST
    let body = if method == "POST" {
        let content_length: usize = header_str
            .lines()
            .find(|l| l.to_lowercase().starts_with("content-length:"))
            .and_then(|l| l.split(':').nth(1))
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(0);
        if content_length > 0 {
            let leftover = &buf[header_end + 4..];
            let mut body = leftover.to_vec();
            if body.len() < content_length {
                let remaining = content_length - body.len();
                let mut rest = vec![0u8; remaining];
                reader.read_exact(&mut rest).await?;
                body.extend_from_slice(&rest);
            }
            body.truncate(content_length);
            String::from_utf8_lossy(&body).to_string()
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    // Route
    let (status, content_type, response_body) = route(method, path, &body, &db);

    // Send response
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nX-Powered-By: NAWA/0.1.0\r\nAccess-Control-Allow-Origin: *\r\n\r\n",
        response_body.len()
    );
    writer.write_all(response.as_bytes()).await?;
    writer.write_all(response_body.as_bytes()).await?;
    writer.flush().await?;

    Ok(())
}

fn route(
    method: &str,
    path: &str,
    body: &str,
    db: &DbEngine,
) -> (String, String, String) {
    let (status, content_type, response_body) = match (method, path) {
        ("GET", "/") => {
            let posts = db.scan_prefix("post:", 100);
            ("200 OK".to_string(), "text/html; charset=utf-8".to_string(), render_home(&posts))
        }
        ("GET", "/health") => {
            let stats = db.stats();
            let json = serde_json::json!({
                "status": "ok",
                "posts": db.len(),
                "stats": { "puts": stats.puts, "gets": stats.gets, "deletes": stats.deletes }
            });
            ("200 OK".into(), "application/json".into(), serde_json::to_string(&json).unwrap())
        }
        ("GET", "/api/posts") => {
            let posts = db.scan_prefix("post:", 100);
            let results: Vec<_> = posts.iter().map(|(k, v)| {
                serde_json::json!({
                    "id": String::from_utf8_lossy(k).strip_prefix("post:").unwrap_or(""),
                    "data": serde_json::from_str::<serde_json::Value>(&v.display()).unwrap_or(serde_json::Value::Null)
                })
            }).collect();
            let json = serde_json::json!({ "posts": results, "count": results.len() });
            ("200 OK".into(), "application/json".into(), serde_json::to_string(&json).unwrap())
        }
        ("POST", "/api/posts") => {
            match serde_json::from_str::<serde_json::Value>(body) {
                Ok(json) => {
                    let id = json["id"].as_str().unwrap_or("untitled").to_string();
                    let key = format!("post:{id}");
                    let value = Value::from_json_str(body).unwrap_or_else(|_| Value::Bytes(body.as_bytes().to_vec()));
                    match db.put(&key, value) {
                        Ok(_) => {
                            let resp = serde_json::json!({ "status": "created", "id": id, "key": key });
                            ("200 OK".into(), "application/json".into(), serde_json::to_string(&resp).unwrap())
                        }
                        Err(e) => {
                            let resp = serde_json::json!({ "error": e.to_string() });
                            ("500 Internal Server Error".into(), "application/json".into(), serde_json::to_string(&resp).unwrap())
                        }
                    }
                }
                Err(e) => {
                    let resp = serde_json::json!({ "error": format!("invalid JSON: {e}") });
                    ("400 Bad Request".into(), "application/json".into(), serde_json::to_string(&resp).unwrap())
                }
            }
        }
        ("GET", p) if p.starts_with("/api/posts/") => {
            let id = p.strip_prefix("/api/posts/").unwrap_or("");
            let key = format!("post:{id}");
            match db.get(&key) {
                Some(v) => {
                    let json = serde_json::json!({
                        "id": id,
                        "data": serde_json::from_str::<serde_json::Value>(&v.display()).unwrap_or(serde_json::Value::Null)
                    });
                    ("200 OK".into(), "application/json".into(), serde_json::to_string(&json).unwrap())
                }
                None => ("404 Not Found".into(), "application/json".into(), r#"{"error":"not found"}"#.to_string()),
            }
        }
        ("DELETE", p) if p.starts_with("/api/posts/") => {
            let id = p.strip_prefix("/api/posts/").unwrap_or("");
            let key = format!("post:{id}");
            match db.delete(&key) {
                Ok(true) => {
                    let resp = serde_json::json!({ "status": "deleted", "id": id });
                    ("200 OK".into(), "application/json".into(), serde_json::to_string(&resp).unwrap())
                }
                _ => ("404 Not Found".into(), "application/json".into(), r#"{"error":"not found"}"#.to_string()),
            }
        }
        ("GET", "/api/stats") => {
            let stats = db.stats();
            let json = serde_json::json!({
                "total_posts": db.len(),
                "db_stats": { "puts": stats.puts, "gets": stats.gets, "deletes": stats.deletes, "scans": stats.scans },
                "memtable_bytes": db.memtable_size()
            });
            ("200 OK".into(), "application/json".into(), serde_json::to_string(&json).unwrap())
        }
        ("GET", "/api") => {
            let json = serde_json::json!({
                "name": "NAWA Blog Site",
                "version": "0.1.0",
                "endpoints": [
                    "GET /", "GET /health", "GET /api/posts", "POST /api/posts",
                    "GET /api/posts/:id", "DELETE /api/posts/:id", "GET /api/stats"
                ]
            });
            ("200 OK".into(), "application/json".into(), serde_json::to_string(&json).unwrap())
        }
        _ => ("404 Not Found".into(), "text/html; charset=utf-8".into(), render_404()),
    };
    (status, content_type, response_body)
}

fn render_home(posts: &[(Vec<u8>, Value)]) -> String {
    let cards: String = posts.iter().map(|(k, v)| {
        let k_str = String::from_utf8_lossy(k);
        let id = k_str.strip_prefix("post:").unwrap_or(&k_str);
        let json: serde_json::Value = serde_json::from_str(&v.display()).unwrap_or(serde_json::json!({}));
        let title = json["title"].as_str().unwrap_or("بدون عنوان");
        let body: String = json["body"].as_str().unwrap_or("").chars().take(120).collect();
        format!(r#"<article class="card"><h2><a href="/api/posts/{id}">{title}</a></h2><p>{body}...</p></article>"#)
    }).collect();

    format!(r#"<!DOCTYPE html><html lang="ar" dir="rtl"><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>مدونة NAWA</title><style>*{{margin:0;padding:0;box-sizing:border-box}}body{{font-family:system-ui,sans-serif;background:#0d0c0a;color:#e0e0e0;line-height:1.8}}.c{{max-width:800px;margin:0 auto;padding:2rem}}header{{text-align:center;padding:3rem 0;border-bottom:1px solid #2a2a2a}}h1{{color:#f59e0b;font-size:2.5rem}}.card{{padding:1.5rem;margin:1rem 0;background:#1a1a1a;border-radius:12px;border:1px solid #2a2a2a}}.card:hover{{border-color:#f59e0b}}.card h2 a{{color:#e0e0e0;text-decoration:none}}.card h2 a:hover{{color:#f59e0b}}.card p{{color:#999;margin:.5rem 0}}nav a{{color:#f59e0b;margin-left:1rem;text-decoration:none}}footer{{text-align:center;padding:2rem;color:#555;border-top:1px solid #2a2a2a;margin-top:3rem}}</style></head><body><div class="c"><header><h1>مدونة NAWA</h1><p>موقع مدونة بُني بـ Rust · يعمل على NAWA</p><nav><a href="/">الرئيسية</a><a href="/api/posts">API</a><a href="/api/stats">Stats</a><a href="/health">Health</a></nav></header><main>{cards}</main><footer><p>© 2026 مدونة NAWA · مبني بـ <strong>NAWA OS</strong></p></footer></div></body></html>"#)
}

fn render_404() -> String {
    r#"<!DOCTYPE html><html><head><meta charset="UTF-8"><title>404</title><style>body{font-family:system-ui;background:#0d0c0a;color:#e0e0e0;display:grid;place-items:center;min-height:100vh}h1{font-size:6rem;color:#f59e0b}a{color:#f59e0b;text-decoration:none}</style></head><body><div style="text-align:center"><h1>404</h1><p>الصفحة غير موجودة</p><a href="/">← العودة للرئيسية</a></div></body></html>"#.to_string()
}

fn seed_sample_posts(db: &DbEngine) -> anyhow::Result<()> {
    let posts = vec![
        serde_json::json!({"id":"1","title":"مرحباً من NAWA","body":"أول مقال في مدونة NAWA. تم تخزينه في قاعدة بيانات NAWA-DB المدمجة.","tags":["nawa","rust"],"author":"فريق NAWA","published_at":"2026-07-06"}),
        serde_json::json!({"id":"2","title":"كيف يعمل io_uring","body":"io_uring هي واجهة I/O غير متزامنة في Linux 5.1+. NAWA يستخدمها لـ zero-copy I/O.","tags":["rust","io_uring"],"author":"فريق NAWA","published_at":"2026-07-06"}),
        serde_json::json!({"id":"3","title":"بناء DB من الصفر","body":"NAWA-DB قاعدة بيانات مكتوبة من الصفر بـ Rust. LSM tree + WAL + Bloom filter.","tags":["rust","database"],"author":"فريق NAWA","published_at":"2026-07-06"}),
    ];
    for post in posts {
        let id = post["id"].as_str().unwrap().to_string();
        db.put(format!("post:{id}"), Value::from_json_str(&serde_json::to_string(&post)?)?)?;
    }
    Ok(())
}
