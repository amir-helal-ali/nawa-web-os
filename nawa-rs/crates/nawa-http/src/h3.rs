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
//! For the alpha, this module provides the structure and config.
//! Full HTTP/3 serving will be enabled in v0.2.0 once h3 + h3-quinn
//! stabilize further.

use std::net::SocketAddr;
use std::sync::Arc;

use crate::router::Router;
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
}

/// HTTP/3 server.
///
/// For the alpha, this struct holds the config and router but doesn't
/// actually serve (HTTP/3 stack is stubbed out). v0.2.0 will wire up
/// `quinn::Endpoint` + `h3_quinn` + `h3::server`.
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

    /// Start serving HTTP/3.
    ///
    /// For the alpha, this returns `NotImplemented`. The real implementation
    /// will use `quinn::Endpoint::bind` + `h3_quinn::Connection` + `h3::server`.
    pub async fn serve(self) -> Result<(), Http3Error> {
        tracing::info!(
            addr = %self.config.addr,
            streams = self.config.max_streams,
            routes = self.route_count(),
            "HTTP/3 server would listen (stub — full impl in v0.2.0)"
        );
        // For now, sleep forever (in production: real QUIC accept loop).
        // The real impl:
        //   let endpoint = quinn::Endpoint::server(self.config.tls, self.config.addr)?;
        //   while let Some(conn) = endpoint.accept().await {
        //       let h3_conn = h3::server::new(h3_quinn::Connection::new(conn.await?)).await?;
        //       // ... handle streams via self.router.dispatch(...)
        //   }
        std::future::pending::<()>().await;
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
    #[error("not implemented in alpha — coming in v0.2.0")]
    NotImplemented,
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn http3_error_display() {
        let e = Http3Error::NotImplemented;
        assert!(format!("{e}").contains("v0.2.0"));
    }

    #[test]
    fn http3_error_quic() {
        let e = Http3Error::Quic("test".into());
        assert!(format!("{e}").contains("QUIC"));
    }
}
