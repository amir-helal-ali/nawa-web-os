//! HTTP/3 + QUIC server (via quinn + h3).
//!
//! HTTP/3 is the next-gen HTTP protocol running over QUIC. Benefits:
//! - No head-of-line blocking (independent streams).
//! - 0-RTT connection resumption.
//! - Built-in TLS 1.3.
//! - Connection migration.

use std::net::SocketAddr;
use std::sync::Arc;

use crate::router::{Method, Request, Response, Router};
use crate::tls::TlsConfig;

/// HTTP/3 server configuration.
#[derive(Clone)]
pub struct Http3Config {
    /// Address to bind on (UDP — QUIC runs over UDP).
    pub addr: SocketAddr,
    /// TLS config (required for HTTP/3 — QUIC mandates TLS 1.3).
    pub tls: Arc<TlsConfig>,
    /// Max concurrent bidirectional streams per connection.
    pub max_streams: u64,
    /// Max idle timeout (ms).
    pub idle_timeout_ms: u64,
    /// Max connection window size (bytes).
    pub max_window: u64,
}

impl std::fmt::Debug for Http3Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Http3Config")
            .field("addr", &self.addr)
            .field("max_streams", &self.max_streams)
            .field("idle_timeout_ms", &self.idle_timeout_ms)
            .field("max_window", &self.max_window)
            .finish()
    }
}

impl Http3Config {
    /// Create a new HTTP/3 config.
    pub fn new(addr: SocketAddr, tls: TlsConfig) -> Self {
        Self {
            addr,
            tls: Arc::new(tls),
            max_streams: 100,
            idle_timeout_ms: 30_000,
            max_window: 1_048_576, // 1 MB
        }
    }

    /// Set max concurrent streams.
    pub fn with_max_streams(mut self, n: u64) -> Self {
        self.max_streams = n;
        self
    }

    /// Set idle timeout.
    pub fn with_idle_timeout(mut self, ms: u64) -> Self {
        self.idle_timeout_ms = ms;
        self
    }
}

/// HTTP/3 server.
///
/// Uses `quinn` for QUIC transport and `h3` + `h3-quinn` for HTTP/3.
pub struct Http3Server {
    config: Http3Config,
    router: Arc<Router>,
}

impl Http3Server {
    /// Create a new HTTP/3 server.
    pub fn new(config: Http3Config, router: Router) -> Self {
        Self {
            config,
            router: Arc::new(router),
        }
    }

    /// Get the server config.
    pub fn config(&self) -> &Http3Config {
        &self.config
    }

    /// Get the bound address.
    pub fn addr(&self) -> SocketAddr {
        self.config.addr
    }

    /// Number of registered routes.
    pub fn route_count(&self) -> usize {
        self.router.len()
    }

    /// Start serving HTTP/3 over QUIC.
    pub async fn serve(self) -> Result<(), Http3Error> {
        tracing::info!(
            addr = %self.config.addr,
            streams = self.config.max_streams,
            routes = self.route_count(),
            "HTTP/3 server starting (QUIC + h3)"
        );

        // Build QUIC server config from TLS config.
        let quinn_server_config = quinn::ServerConfig::with_crypto(self.config.tls.config());
        let endpoint = quinn::Endpoint::server(quinn_server_config, self.config.addr)
            .map_err(|e| Http3Error::Quic(e.to_string()))?;

        tracing::info!("HTTP/3 server listening on {}", endpoint.local_addr()?);

        // Accept loop.
        while let Some(incoming) = endpoint.accept().await {
            let router = self.router.clone();
            tokio::spawn(async move {
                match incoming.await {
                    Ok(conn) => {
                        if let Err(e) = handle_h3_connection(conn, router).await {
                            tracing::debug!("h3 connection error: {e}");
                        }
                    }
                    Err(e) => {
                        tracing::debug!("quic accept error: {e}");
                    }
                }
            });
        }

        Ok(())
    }
}

/// Handle a single HTTP/3 connection.
async fn handle_h3_connection(
    conn: quinn::Connection,
    router: Arc<Router>,
) -> Result<(), Http3Error> {
    let mut h3_conn = h3::server::Connection::new(h3_quinn::Connection::new(conn))
        .await
        .map_err(|e| Http3Error::H3(e.to_string()))?;

    loop {
        match h3_conn.accept().await {
            Ok(Some((req_stream, req))) => {
                let router = router.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle_h3_request(req_stream, req, router).await {
                        tracing::debug!("h3 request error: {e}");
                    }
                });
            }
            Ok(None) => break,
            Err(e) => {
                tracing::debug!("h3 accept error: {e}");
                break;
            }
        }
    }

    Ok(())
}

/// Handle a single HTTP/3 request.
async fn handle_h3_request(
    mut req_stream: h3::server::RequestStream<mut h3_quinn::BidiStream<bytes::Bytes>>,
    req: http::Request<()>,
    router: Arc<Router>,
) -> Result<(), Http3Error> {
    let method = match req.method().as_str() {
        "GET" => Method::Get,
        "POST" => Method::Post,
        "PUT" => Method::Put,
        "DELETE" => Method::Delete,
        "PATCH" => Method::Patch,
        "HEAD" => Method::Head,
        "OPTIONS" => Method::Options,
        _ => Method::Get,
    };

    let path = req.uri().path().to_string();
    let query: std::collections::HashMap<String, String> = req
        .uri()
        .query()
        .map(|q| {
            q.split('&')
                .filter_map(|kv| {
                    let (k, v) = kv.split_once('=')?;
                    Some((k.to_string(), v.to_string()))
                })
                .collect()
        })
        .unwrap_or_default();

    let headers: std::collections::HashMap<String, String> = req
        .headers()
        .iter()
        .map(|(k, v)| (k.as_str().to_lowercase(), v.to_str().unwrap_or("").to_string()))
        .collect();

    let mut body = Vec::new();
    while let Some(chunk) = req_stream.recv_data().await.map_err(|e| Http3Error::H3(e.to_string()))? {
        body.extend_from_slice(&chunk);
    }

    let nawa_req = Request {
        method,
        path,
        query,
        headers,
        body,
        params: std::collections::HashMap::new(),
    };

    let resp = router.dispatch(nawa_req).await;

    let status_code = resp.status.0;
    let response = http::Response::builder()
        .status(status_code)
        .header("content-type", "text/plain")
        .header("content-length", resp.body.len())
        .header("x-powered-by", "NAWA/0.1.0 HTTP/3")
        .body(())
        .map_err(|e| Http3Error::H3(e.to_string()))?;

    req_stream
        .send_response(response)
        .await
        .map_err(|e| Http3Error::H3(e.to_string()))?;

    if !resp.body.is_empty() {
        req_stream
            .send_data(bytes::Bytes::from(resp.body))
            .await
            .map_err(|e| Http3Error::H3(e.to_string()))?;
    }

    req_stream
        .finish()
        .await
        .map_err(|e| Http3Error::H3(e.to_string()))?;

    Ok(())
}

/// HTTP/3 error type.
#[derive(Debug, thiserror::Error)]
pub enum Http3Error {
    #[error("QUIC error: {0}")]
    Quic(String),
    #[error("H3 error: {0}")]
    H3(String),
    #[error("TLS error: {0}")]
    Tls(#[from] crate::TlsError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn http3_error_display() {
        let e = Http3Error::Quic("test".into());
        assert!(format!("{e}").contains("QUIC"));
    }

    #[test]
    fn http3_error_h3() {
        let e = Http3Error::H3("parse error".into());
        assert!(format!("{e}").contains("H3"));
    }
}
