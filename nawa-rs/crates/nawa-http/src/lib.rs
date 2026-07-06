//! # NAWA HTTP
//!
//! HTTP/1.1 + HTTP/3 server with a type-safe router.

pub mod acme;
pub mod h3;
pub mod router;
pub mod server;
pub mod tls;

pub use acme::{AcmeClient, AcmeConfig, AcmeError};
pub use h3::{Http3Config, Http3Error, Http3Server};
pub use router::{Handler, Method, Request, Response, Router, StatusCode};
pub use server::HttpServer;
pub use tls::{TlsConfig, TlsError};

/// HTTP error type.
#[derive(Debug, thiserror::Error)]
pub enum HttpError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("parse error: {0}")]
    Parse(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("tls error: {0}")]
    Tls(#[from] TlsError),
}

pub type Result<T> = std::result::Result<T, HttpError>;
