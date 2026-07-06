//! HTTP/1.1 server over TCP.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::net::{TcpListener, TcpStream};

use crate::router::{Method, Request, Response, Router};
use crate::Result;

/// The HTTP server.
pub struct HttpServer {
    router: Arc<Router>,
    addr: SocketAddr,
}

impl HttpServer {
    pub fn new(router: Router, addr: SocketAddr) -> Self {
        Self {
            router: Arc::new(router),
            addr,
        }
    }

    /// Bind and serve forever.
    pub async fn serve(self) -> Result<()> {
        let listener = TcpListener::bind(self.addr).await?;
        tracing::info!("NAWA HTTP server listening on {}", self.addr);
        loop {
            let (stream, peer) = match listener.accept().await {
                Ok(s) => s,
                Err(e) => {
                    tracing::error!("accept failed: {e}");
                    continue;
                }
            };
            let router = self.router.clone();
            tokio::spawn(async move {
                if let Err(e) = handle_connection(stream, router).await {
                    tracing::debug!("connection from {peer} closed: {e}");
                }
            });
        }
    }
}

async fn handle_connection(stream: TcpStream, router: Arc<Router>) -> Result<()> {
    let (read_half, write_half) = stream.into_split();
    let mut reader = BufReader::new(read_half);
    let mut writer = BufWriter::new(write_half);

    loop {
        let req = match read_request(&mut reader).await? {
            Some(r) => r,
            None => return Ok(()), // client closed
        };
        let started = std::time::Instant::now();
        let resp = router.dispatch(req).await;
        let elapsed = started.elapsed();
        write_response(&mut writer, &resp, elapsed).await?;
        writer.flush().await?;
    }
}

async fn read_request<R: AsyncReadExt + Unpin>(
    reader: &mut R,
) -> Result<Option<Request>> {
    // Read until we have the headers.
    let mut buf = Vec::new();
    let mut tmp = [0u8; 1024];
    let mut header_end: Option<usize> = None;
    loop {
        let n = reader.read(&mut tmp).await?;
        if n == 0 {
            if buf.is_empty() {
                return Ok(None);
            }
            break;
        }
        let prev_len = buf.len();
        buf.extend_from_slice(&tmp[..n]);
        if header_end.is_none() {
            if let Some(end) = find_double_crlf(&buf[prev_len.saturating_sub(3)..]) {
                header_end = Some(prev_len.saturating_sub(3) + end + 4);
                break;
            } else if let Some(end) = find_double_crlf(&buf) {
                header_end = Some(end + 4);
                break;
            }
        }
        if buf.len() > 64 * 1024 {
            return Err(crate::HttpError::Parse("headers too large".into()));
        }
    }

    let header_end = header_end.ok_or_else(|| crate::HttpError::Parse("no header end".into()))?;
    let header_bytes = &buf[..header_end];
    let leftover = &buf[header_end..];

    let headers_str = std::str::from_utf8(header_bytes).map_err(|e| crate::HttpError::Parse(e.to_string()))?;
    let mut lines = headers_str.split("\r\n");
    let request_line = lines
        .next()
        .ok_or_else(|| crate::HttpError::Parse("no request line".into()))?;
    let mut parts = request_line.split_whitespace();
    let method_str = parts
        .next()
        .ok_or_else(|| crate::HttpError::Parse("no method".into()))?;
    let path_with_query = parts
        .next()
        .ok_or_else(|| crate::HttpError::Parse("no path".into()))?;

    let method = Method::from_str(method_str)
        .ok_or_else(|| crate::HttpError::Parse(format!("unknown method: {method_str}")))?;

    let (path, query) = split_path_query(path_with_query);
    let query = parse_query(&query);

    let mut headers = HashMap::new();
    for line in lines {
        if line.is_empty() {
            break;
        }
        if let Some((k, v)) = line.split_once(':') {
            headers.insert(k.trim().to_lowercase(), v.trim().to_string());
        }
    }

    // Read body if Content-Length is present.
    let content_length: usize = headers
        .get("content-length")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let body = if content_length > 0 {
        let mut body = vec![0u8; content_length];
        // First copy any leftover bytes we already read.
        let to_copy = leftover.len().min(content_length);
        body[..to_copy].copy_from_slice(&leftover[..to_copy]);
        if to_copy < content_length {
            reader.read_exact(&mut body[to_copy..]).await?;
        }
        body
    } else {
        Vec::new()
    };

    Ok(Some(Request {
        method,
        path,
        query,
        headers,
        body,
        params: HashMap::new(),
    }))
}

async fn write_response<W: AsyncWriteExt + Unpin>(
    writer: &mut W,
    resp: &Response,
    elapsed: std::time::Duration,
) -> Result<()> {
    let status_line = format!("HTTP/1.1 {} {}\r\n", resp.status.0, resp.status.reason_phrase());
    writer.write_all(status_line.as_bytes()).await?;

    let mut has_content_length = false;
    let mut has_content_type = false;
    for (k, v) in &resp.headers {
        if k.eq_ignore_ascii_case("content-length") {
            has_content_length = true;
        }
        if k.eq_ignore_ascii_case("content-type") {
            has_content_type = true;
        }
        writer.write_all(format!("{k}: {v}\r\n").as_bytes()).await?;
    }
    if !has_content_type {
        writer
            .write_all(b"Content-Type: application/octet-stream\r\n")
            .await?;
    }
    if !has_content_length {
        writer
            .write_all(format!("Content-Length: {}\r\n", resp.body.len()).as_bytes())
            .await?;
    }
    writer
        .write_all(format!("X-Response-Time: {}μs\r\n", elapsed.as_micros()).as_bytes())
        .await?;
    writer.write_all(b"X-Powered-By: NAWA/0.1.0\r\n").await?;
    writer.write_all(b"\r\n").await?;
    writer.write_all(&resp.body).await?;
    Ok(())
}

fn find_double_crlf(buf: &[u8]) -> Option<usize> {
    buf.windows(4).position(|w| w == b"\r\n\r\n")
}

fn split_path_query(s: &str) -> (String, String) {
    if let Some((p, q)) = s.split_once('?') {
        (p.to_string(), q.to_string())
    } else {
        (s.to_string(), String::new())
    }
}

fn parse_query(q: &str) -> HashMap<String, String> {
    if q.is_empty() {
        return HashMap::new();
    }
    q.split('&')
        .filter_map(|kv| {
            let (k, v) = kv.split_once('=')?;
            Some((url_decode(k), url_decode(v)))
        })
        .collect()
}

fn url_decode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '%' {
            let hex: String = chars.by_ref().take(2).collect();
            if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                out.push(byte as char);
            }
        } else if c == '+' {
            out.push(' ');
        } else {
            out.push(c);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_path_query_basic() {
        let (p, q) = split_path_query("/users?id=42");
        assert_eq!(p, "/users");
        assert_eq!(q, "id=42");
    }

    #[test]
    fn parse_query_basic() {
        let q = parse_query("a=1&b=2&c=hello%20world");
        assert_eq!(q.get("a"), Some(&"1".to_string()));
        assert_eq!(q.get("b"), Some(&"2".to_string()));
        assert_eq!(q.get("c"), Some(&"hello world".to_string()));
    }

    #[test]
    fn find_double_crlf_works() {
        assert_eq!(find_double_crlf(b"GET / HTTP/1.1\r\n\r\n"), Some(14));
        assert_eq!(find_double_crlf(b"no crlf here"), None);
    }
}
