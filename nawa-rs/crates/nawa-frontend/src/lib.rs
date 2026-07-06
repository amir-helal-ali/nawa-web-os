//! # NAWA Frontend Engine
//!
//! Real SSR renderer with island hydration architecture.
//!
//! ## Components
//!
//! - `html` — Type-safe HTML builder (Rust → HTML)
//! - `island` — Island hydration system (interactive components)
//! - `stream` — Streaming SSR with Suspense
//! - `template` — Template system for pages

pub mod html;
pub mod island;
pub mod stream;
pub mod template;

pub use html::{Html, HtmlElement, HtmlNode};
pub use island::{Island, IslandRegistry, IslandProps};
pub use stream::{StreamingResponse, Suspense, SuspenseResult};
pub use template::{PageTemplate, TemplateBuilder};

/// Frontend error type.
#[derive(Debug, thiserror::Error)]
pub enum FrontendError {
    #[error("render error: {0}")]
    Render(String),
    #[error("island error: {0}")]
    Island(String),
    #[error("template error: {0}")]
    Template(String),
}

pub type Result<T> = std::result::Result<T, FrontendError>;
