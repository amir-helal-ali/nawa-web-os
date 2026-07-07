//! # NAWA-Svelte — SvelteKit integration for NAWA Web Operating System
//!
//! Embeds a SvelteKit application inside the NAWA Rust binary.
//! No Node.js required at runtime — SvelteKit is compiled at build time
//! via `adapter-nawa`, producing a `_nawa/` directory containing:
//!
//! - `manifest.json` — route table + metadata
//! - `pages/*.html` — pre-rendered HTML
//! - `assets/*.{js,css}` — hydration bundles
//! - `ssr/*.wasm` — (future) SSR WASM modules
//!
//! ## Architecture
//!
//! ```text
//! ┌──────────── Build-time (Node.js) ────────────┐
//! │  SvelteKit + adapter-nawa                    │
//! │  → produces _nawa/ directory                 │
//! └────────────────────┬─────────────────────────┘
//!                      ↓ embed in binary
//! ┌──────────── Runtime (Rust) ──────────────────┐
//! │  nawa-svelte:                                │
//! │  • manifest.rs — parse + route matching      │
//! │  • renderer.rs — HTML rendering + bootstrap  │
//! │  • handler.rs  — HTTP request handling       │
//! └──────────────────────────────────────────────┘
//! ```
//!
//! ## Usage
//!
//! ```no_run
//! use nawa_svelte::SvelteHandler;
//! let handler = SvelteHandler::load("./svelte-app/_nawa", "ws://localhost:8081").unwrap();
//! let page = handler.handle("/", Default::default(), None, None, serde_json::Value::Null);
//! ```

pub mod handler;
pub mod manifest;
pub mod renderer;

pub use handler::SvelteHandler;
pub use manifest::{MatchedRoute, MetaTags, NawaManifest, Route};
pub use renderer::{RenderContext, RenderedPage, SvelteRenderer};

/// Error type for the SvelteKit integration.
#[derive(Debug, thiserror::Error)]
pub enum SvelteError {
    #[error("manifest error: {0}")]
    Manifest(String),
    #[error("render error: {0}")]
    Render(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, SvelteError>;
