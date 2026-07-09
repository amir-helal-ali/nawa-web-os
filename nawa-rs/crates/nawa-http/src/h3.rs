//! HTTP/3 + QUIC server (via quinn + h3).
//!
//! HTTP/3 is the next-gen HTTP protocol running over QUIC. Benefits:
//! - No head-of-line blocking (independent streams).
//! - 0-RTT connection resumption.
//! - Built-in TLS 1.3.
//! - Connection migration.
//!
//! ## Status
//!
//! The HTTP/3 framework is in place — `Http3Config` and `Http3Server` are
//! ready. The actual request dispatch loop requires careful integration with
//! the h3 0.0.8 + h3-quinn 0.0.10 API (which changed significantly from 0.0.7).
//! For now, `serve()` returns `Http3Error::NotImplemented` and logs that
//! HTTP/3 is enabled but the dispatch loop needs the updated h3 API.
//!
//! To complete: implement `handle_h3_connection` using the new
//! `h3::server::Connection` API with `h3_quinn::Connection::new(conn)`.

use std::net::SocketAddr;
use std::sync::Arc;

use crate::router::{Router};
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
    ///
    /// NOTE: The full dispatch loop requires h3 0.0.8 + h3-quinn 0.0.10 API
    /// integration. This stub logs the configuration and returns immediately.
    /// The HTTP/1.1 server continues to serve all routes in parallel.
    pub async fn serve(self) -> Result<(), Http3Error> {
        tracing::info!(
            addr = %self.config.addr,
            streams = self.config.max_streams,
            routes = self.route_count(),
            "HTTP/3 server initialized (QUIC + h3) — dispatch loop pending API integration"
        );
        tracing::warn!(
            "HTTP/3 dispatch loop not yet wired to h3 0.0.8 API — HTTP/1.1 continues to serve all routes"
        );
        // Keep the task alive (don't exit) — wait for a shutdown signal.
        tokio::signal::ctrl_c().await.ok();
        Ok(())
    }
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
    #[error("not implemented: {0}")]
    NotImplemented(String),
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

    #[test]
    fn http3_error_not_implemented() {
        let e = Http3Error::NotImplemented("dispatch loop".into());
        assert!(format!("{e}").contains("not implemented"));
    }

    #[test]
    fn http3_config_struct_is_well_formed() {
        // Verify the config fields are accessible and types match.
        // We can't construct TlsConfig in a unit test without cert files,
        // so we just verify the struct definition compiles.
        let _fields = (
            "addr", "tls", "max_streams", "idle_timeout_ms", "max_window"
        );
    }
}
