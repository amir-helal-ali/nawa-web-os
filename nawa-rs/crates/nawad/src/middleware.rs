//! Built-in middleware — rate limiting + security headers + static files.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
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
        Self {
            max_requests,
            window,
            clients: Mutex::new(HashMap::new()),
        }
    }

    #[allow(dead_code)]
    pub fn check(&self, ip: &str) -> bool {
        let mut clients = self.clients.lock().unwrap();
        let now = Instant::now();
        let entry = clients.entry(ip.to_string()).or_insert((0, now));
        if now.duration_since(entry.1) > self.window {
            *entry = (1, now);
            return true;
        }
        entry.0 += 1;
        entry.0 <= self.max_requests
    }
}

/// Security headers to add to every response.
pub const SECURITY_HEADERS: &[(&str, &str)] = &[
    ("X-Content-Type-Options", "nosniff"),
    ("X-Frame-Options", "DENY"),
    ("X-XSS-Protection", "1; mode=block"),
    ("Referrer-Policy", "strict-origin-when-cross-origin"),
    ("Permissions-Policy", "geolocation=(), microphone=(), camera=()"),
];

/// Add security headers to a response.
pub fn add_security_headers(resp: &mut nawa_http::Response) {
    for (k, v) in SECURITY_HEADERS {
        resp.header(k, v);
    }
}

/// Static file server — serves files from a directory.
pub struct StaticServer {
    root: PathBuf,
}

impl StaticServer {
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self { root: root.as_ref().to_path_buf() }
    }

    /// Try to serve a static file. Returns Some(response) if found, None if not.
    pub fn serve(&self, path: &str) -> Option<nawa_http::Response> {
        // Security: prevent path traversal.
        if path.contains("..") || path.contains('\0') {
            return None;
        }

        let file_path = self.root.join(path.trim_start_matches('/'));
        if !file_path.exists() || !file_path.is_file() {
            return None;
        }

        let content = std::fs::read(&file_path).ok()?;
        let content_type = mime_type(&file_path);

        let mut resp = nawa_http::Response::ok(content);
        resp.header("Content-Type", content_type);
        resp.header("Cache-Control", "public, max-age=3600");
        add_security_headers(&mut resp);
        Some(resp)
    }
}

/// Get MIME type from file extension.
fn mime_type(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html") | Some("htm") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "application/javascript; charset=utf-8",
        Some("json") => "application/json; charset=utf-8",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
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

/// Extract client IP from request (simplified — uses peer addr in production).
#[allow(dead_code)]
pub fn extract_ip(headers: &HashMap<String, String>) -> String {
    headers
        .get("x-forwarded-for")
        .and_then(|h| h.split(',').next())
        .map(|s| s.trim().to_string())
        .or_else(|| headers.get("x-real-ip").map(|s| s.clone()))
        .unwrap_or_else(|| "127.0.0.1".to_string())
}

/// Extract a cookie value from a Cookie header string.
pub fn extract_cookie_value(cookie_header: &str, name: &str) -> Option<String> {
    let header_value = if cookie_header.to_lowercase().starts_with("cookie:") {
        cookie_header[7..].trim()
    } else {
        cookie_header.trim()
    };
    header_value
        .split(';')
        .find_map(|c| {
            let c = c.trim();
            let prefix = format!("{name}=");
            c.strip_prefix(&prefix).map(|v| v.to_string())
        })
        .filter(|v| !v.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rate_limiter_allows() {
        let rl = RateLimiter::new(5, Duration::from_secs(60));
        for _ in 0..5 { assert!(rl.check("1.1.1.1")); }
    }

    #[test]
    fn rate_limiter_blocks() {
        let rl = RateLimiter::new(3, Duration::from_secs(60));
        for _ in 0..3 { rl.check("1.1.1.1"); }
        assert!(!rl.check("1.1.1.1"));
    }

    #[test]
    fn static_server_serves() {
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
        assert!(ss.serve("/missing.txt").is_none());
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
        assert_eq!(mime_type(Path::new("a.js")), "application/javascript; charset=utf-8");
        assert_eq!(mime_type(Path::new("a.png")), "image/png");
        assert_eq!(mime_type(Path::new("a.wasm")), "application/wasm");
        assert_eq!(mime_type(Path::new("a.unknown")), "application/octet-stream");
    }

    #[test]
    fn security_headers_added() {
        let mut resp = nawa_http::Response::text("hello");
        add_security_headers(&mut resp);
        assert_eq!(resp.headers.get("x-content-type-options"), Some(&"nosniff".to_string()));
        assert_eq!(resp.headers.get("x-frame-options"), Some(&"DENY".to_string()));
    }
}
