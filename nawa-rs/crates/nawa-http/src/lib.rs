//! # NAWA HTTP
//!
//! HTTP/1.1 server with a type-safe router. For alpha, we use HTTP/1.1
//! over TCP. HTTP/3 (QUIC) will land in v0.2.0.

pub mod router;
pub mod server;

pub use router::{Handler, Method, Request, Response, Router, StatusCode};
pub use server::HttpServer;

/// HTTP error type.
#[derive(Debug, thiserror::Error)]
pub enum HttpError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("parse error: {0}")]
    Parse(String),
    #[error("not found: {0}")]
    NotFound(String),
}

pub type Result<T> = std::result::Result<T, HttpError>;
