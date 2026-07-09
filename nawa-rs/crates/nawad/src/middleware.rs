//! Built-in middleware — rate limiting, security headers, static files,
//! CORS, request logging, auth helpers.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// Simple IP-based rate limiter.
#[allow(dead_code)]
pub struct RateLimiter {
    max_requests: u32,
    window: Duration,
    clients: Mutex<HashMap<String, (u32, Instant)>>,
}

impl RateLimiter {
    pub fn new(max_requests: u32, window: Duration) -> Self {
        Self { max_requests, window, clients: Mutex::new(HashMap::new()) }
    }
    #[allow(dead_code)]
    pub fn check(&self, ip: &str) -> bool {
        let mut c = self.clients.lock().unwrap();
        let now = Instant::now();
        let e = c.entry(ip.to_string()).or_insert((0, now));
        if now.duration_since(e.1) > self.window { *e = (1, now); return true; }
        e.0 += 1;
        e.0 <= self.max_requests
    }
}

/// Security headers to add to every response.
pub const SECURITY_HEADERS: &[(&str, &str)] = &[
    ("X-Content-Type-Options", "nosniff"),
    ("X-Frame-Options", "DENY"),
    ("X-XSS-Protection", "1; mode=block"),
    ("Referrer-Policy", "strict-origin-when-cross-origin"),
    ("Permissions-Policy", "geolocation=(), microphone=(), camera=(), payment=()"),
    ("Content-Security-Policy",
     "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; font-src 'self' data:; connect-src 'self' ws: wss:; frame-ancestors 'none'; base-uri 'self'; form-action 'self';"),
    ("Strict-Transport-Security", "max-age=31536000; includeSubDomains; preload"),
    ("Cross-Origin-Opener-Policy", "same-origin"),
    ("Cross-Origin-Resource-Policy", "same-origin"),
    ("X-Permitted-Cross-Domain-Policies", "none"),
];

/// CSRF token generation and validation.
#[allow(dead_code)]
pub struct CsrfProtection;

impl CsrfProtection {
    /// Generate a new CSRF token (32 random hex chars).
    pub fn generate_token() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.subsec_nanos())
            .unwrap_or(0);
        let pid = std::process::id();
        let mut hash = xxhash_rust::xxh3::xxh3_64(
            format!("{nanos}{pid}{}{}", nanos.wrapping_mul(2654435761), pid).as_bytes()
        );
        hash = hash.wrapping_mul(0x9E3779B97F4A7C15);
        format!("{:016x}{:016x}", hash, hash.rotate_left(32))
    }

    /// Validate a CSRF token (constant-time comparison).
    #[allow(dead_code)]
    pub fn validate_token(token: &str, expected: &str) -> bool {
        if token.len() != expected.len() { return false; }
        let mut diff: u8 = 0;
        for (a, b) in token.bytes().zip(expected.bytes()) { diff |= a ^ b; }
        diff == 0
    }
}

/// Audit log entry.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuditEntry {
    pub timestamp: String,
    pub action: String,
    pub user: Option<String>,
    pub ip: String,
    pub path: String,
    pub method: String,
    pub status: u16,
    pub details: Option<String>,
}

/// Audit logger — stores security events in NAWA-DB.
pub struct AuditLogger;

impl AuditLogger {
    #[allow(clippy::too_many_arguments)]
    #[allow(dead_code)]
    pub fn log(
        db: &nawa_db::DbEngine,
        action: &str,
        user: Option<&str>,
        ip: &str,
        path: &str,
        method: &str,
        status: u16,
        details: Option<&str>,
    ) {
        let entry = AuditEntry {
            timestamp: chrono::Utc::now().to_rfc3339(),
            action: action.to_string(),
            user: user.map(|s| s.to_string()),
            ip: ip.to_string(),
            path: path.to_string(),
            method: method.to_string(),
            status,
            details: details.map(|s| s.to_string()),
        };
        if let Ok(json) = serde_json::to_string(&entry) {
            let key = format!("audit:{}:{}", entry.timestamp, action);
            let _ = db.put(key, nawa_db::Value::from_str(&json));
        }
    }

    pub fn recent(db: &nawa_db::DbEngine, limit: usize) -> Vec<AuditEntry> {
        let entries = db.scan_prefix("audit:", limit);
        let mut audit_entries: Vec<AuditEntry> = entries
            .into_iter()
            .filter_map(|(_, v)| {
                let s = v.display();
                serde_json::from_str(&s).ok()
            })
            .collect();
        audit_entries.reverse();
        audit_entries
    }
}

pub fn add_security_headers(resp: &mut nawa_http::Response) {
    for (k, v) in SECURITY_HEADERS { resp.header(k, v); }
}

/// CORS configuration — allows cross-origin requests to the API.
#[allow(dead_code)]
pub fn add_cors_headers(resp: &mut nawa_http::Response) {
    resp.header("Access-Control-Allow-Origin", "*");
    resp.header("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS");
    resp.header("Access-Control-Allow-Headers", "Content-Type, Authorization, Cookie");
    resp.header("Access-Control-Max-Age", "3600");
}

/// Check if a request is a CORS preflight (OPTIONS).
#[allow(dead_code)]
pub fn is_preflight(method: &str) -> bool {
    method == "OPTIONS"
}

/// Request logger — tracks request count and logs each request.
#[allow(dead_code)]
pub struct RequestLogger {
    total_requests: AtomicU64,
    total_errors: AtomicU64,
}

impl RequestLogger {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self { total_requests: AtomicU64::new(0), total_errors: AtomicU64::new(0) }
    }
    #[allow(dead_code)]
    pub fn log_request(&self, method: &str, path: &str, status: u16) {
        let count = self.total_requests.fetch_add(1, Ordering::Relaxed) + 1;
        if status >= 400 {
            self.total_errors.fetch_add(1, Ordering::Relaxed);
        }
        tracing::info!("[{count}] {method} {path} → {status}");
    }
    #[allow(dead_code)]
    pub fn total(&self) -> u64 { self.total_requests.load(Ordering::Relaxed) }
    #[allow(dead_code)]
    pub fn errors(&self) -> u64 { self.total_errors.load(Ordering::Relaxed) }
}

/// Static file server.
pub struct StaticServer {
    root: PathBuf,
}

impl StaticServer {
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self { root: root.as_ref().to_path_buf() }
    }
    pub fn serve(&self, path: &str) -> Option<nawa_http::Response> {
        if path.contains("..") || path.contains('\0') { return None; }
        let file_path = self.root.join(path.trim_start_matches('/'));
        if !file_path.exists() || !file_path.is_file() { return None; }
        let content = std::fs::read(&file_path).ok()?;
        let ct = mime_type(&file_path);
        let mut resp = nawa_http::Response::ok(content);
        resp.header("Content-Type", ct);
        resp.header("Cache-Control", "public, max-age=3600");
        add_security_headers(&mut resp);
        Some(resp)
    }
}

fn mime_type(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html")|Some("htm") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "application/javascript; charset=utf-8",
        Some("json") => "application/json; charset=utf-8",
        Some("png") => "image/png",
        Some("jpg")|Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        Some("ico") => "image/x-icon",
        Some("woff") => "font/woff",
        Some("woff2") => "font/woff2",
        Some("ttf") => "font/ttf",
        Some("wasm") => "application/wasm",
        Some("txt") => "text/plain; charset=utf-8",
        Some("xml") => "application/xml",
        Some("pdf") => "application/pdf",
        _ => "application/octet-stream",
    }
}

/// Extract a cookie value from a Cookie header.
pub fn extract_cookie_value(cookie_header: &str, name: &str) -> Option<String> {
    let hv = if cookie_header.to_lowercase().starts_with("cookie:") {
        cookie_header[7..].trim()
    } else { cookie_header.trim() };
    hv.split(';').find_map(|c| {
        let c = c.trim();
        let p = format!("{name}=");
        c.strip_prefix(&p).map(|v| v.to_string())
    }).filter(|v| !v.is_empty())
}

/// Extract client IP from request headers.
#[allow(dead_code)]
pub fn extract_ip(headers: &HashMap<String, String>) -> String {
    headers.get("x-forwarded-for")
        .and_then(|h| h.split(',').next())
        .map(|s| s.trim().to_string())
        .or_else(|| headers.get("x-real-ip").cloned())
        .unwrap_or_else(|| "127.0.0.1".to_string())
}

/// Database backup — exports all keys as JSON.
pub fn backup_db(db: &nawa_db::DbEngine) -> Vec<u8> {
    let entries = db.scan_prefix("", 1_000_000);
    let backup: Vec<serde_json::Value> = entries.iter().map(|(k, v)| {
        serde_json::json!({
            "key": String::from_utf8_lossy(k),
            "value": v.display(),
        })
    }).collect();
    let json = serde_json::json!({
        "version": "2.5.0",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "count": backup.len(),
        "entries": backup,
    });
    serde_json::to_vec_pretty(&json).unwrap_or_default()
}

/// Database restore — imports keys from JSON backup.
pub fn restore_db(db: &nawa_db::DbEngine, json: &[u8]) -> Result<usize, String> {
    let data: serde_json::Value = serde_json::from_slice(json).map_err(|e| e.to_string())?;
    let entries = data["entries"].as_array().ok_or("invalid backup format")?;
    let mut count = 0;
    for entry in entries {
        let key = entry["key"].as_str().ok_or("missing key")?;
        let value_str = entry["value"].as_str().ok_or("missing value")?;
        // Try JSON, fall back to bytes.
        let value = if value_str.trim_start().starts_with('{') || value_str.trim_start().starts_with('[') {
            nawa_db::Value::from_json_str(value_str).unwrap_or_else(|_| nawa_db::Value::from_str(value_str))
        } else {
            nawa_db::Value::from_str(value_str)
        };
        db.put(key, value).map_err(|e| e.to_string())?;
        count += 1;
    }
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rate_limiter_works() {
        let rl = RateLimiter::new(3, Duration::from_secs(60));
        for _ in 0..3 { assert!(rl.check("1.1.1.1")); }
        assert!(!rl.check("1.1.1.1"));
    }

    #[test]
    fn static_server_works() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("test.txt"), "hello").unwrap();
        let ss = StaticServer::new(tmp.path());
        let resp = ss.serve("/test.txt").unwrap();
        assert_eq!(resp.body, b"hello");
    }

    #[test]
    fn static_server_404() {
        let tmp = tempfile::tempdir().unwrap();
        let ss = StaticServer::new(tmp.path());
        assert!(ss.serve("/missing").is_none());
    }

    #[test]
    fn static_server_blocks_traversal() {
        let tmp = tempfile::tempdir().unwrap();
        let ss = StaticServer::new(tmp.path());
        assert!(ss.serve("../../../etc/passwd").is_none());
    }

    #[test]
    fn mime_types() {
        assert_eq!(mime_type(Path::new("a.css")), "text/css; charset=utf-8");
        assert_eq!(mime_type(Path::new("a.wasm")), "application/wasm");
        assert_eq!(mime_type(Path::new("a.png")), "image/png");
    }

    #[test]
    fn security_headers() {
        let mut resp = nawa_http::Response::text("hi");
        add_security_headers(&mut resp);
        // Headers are stored with the exact case used in SECURITY_HEADERS.
        assert_eq!(resp.headers.get("X-Content-Type-Options"), Some(&"nosniff".to_string()));
    }

    #[test]
    fn cors_headers() {
        let mut resp = nawa_http::Response::text("hi");
        add_cors_headers(&mut resp);
        assert!(resp.headers.contains_key("Access-Control-Allow-Origin"));
    }

    #[test]
    fn request_logger() {
        let rl = RequestLogger::new();
        rl.log_request("GET", "/", 200);
        rl.log_request("POST", "/data", 500);
        assert_eq!(rl.total(), 2);
        assert_eq!(rl.errors(), 1);
    }

    #[test]
    fn cookie_extraction() {
        let cookie = "Cookie: nawa_token=abc123; other=val";
        assert_eq!(extract_cookie_value(cookie, "nawa_token"), Some("abc123".into()));
        assert_eq!(extract_cookie_value(cookie, "missing"), None);
    }

    #[test]
    fn backup_restore_roundtrip() {
        let db = nawa_db::DbEngine::open_in_memory();
        db.put("key1", nawa_db::Value::from_str("val1")).unwrap();
        db.put("key2", nawa_db::Value::from_str("val2")).unwrap();
        
        let backup = backup_db(&db);
        assert!(!backup.is_empty());
        
        let db2 = nawa_db::DbEngine::open_in_memory();
        let count = restore_db(&db2, &backup).unwrap();
        assert_eq!(count, 2);
        assert_eq!(db2.get("key1").unwrap().display(), "val1");
        assert_eq!(db2.get("key2").unwrap().display(), "val2");
    }
}
