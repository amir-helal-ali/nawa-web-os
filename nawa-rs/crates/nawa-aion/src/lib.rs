//! # AION Engine — Adaptive Intelligent Ontological Network
//!
//! Revolutionary SEO system for NAWA Web Operating System.
//!
//! AION transforms NAWA from a "page server" into a "knowledge provider":
//! - **Ontological Engine** infers entity types from DB structure
//! - **Adaptive Negotiation** picks the best format for each crawler
//! - **Multi-format Renderers** produce 9 response formats
//! - **Photon Protocol** exposes the entire Knowledge Graph in one response
//! - **Self-Healing Loop** auto-fixes SEO issues (planned)
//! - **Predictive Caching** pre-warms likely-crawled URLs (planned)
//!
//! ## Architecture
//!
//! ```text
//! ┌────────────── Runtime (Rust binary) ──────────────────┐
//! │  AION:                                                 │
//! │  • ontology.rs  — infer types + build Knowledge Graph │
//! │  • negotiation.rs — pick format per crawler           │
//! │  • photon.rs    — single endpoint for crawlers        │
//! └────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Endpoints
//!
//! - `GET /__photon__` — Photon Protocol response (entire Knowledge Graph)
//! - `GET /sitemap.xml` — dynamic sitemap from DB
//! - `GET /robots.txt` — robots with AI crawler allowlist
//! - Any entity URL → auto-negotiated format based on User-Agent

pub mod google_search_console;
pub mod healing;
pub mod negotiation;
pub mod ontology;
pub mod photon;

pub use google_search_console::{
    CrawlErrorCount, CrawlErrorCounts, GoogleSearchConsoleClient,
    IndexStatusResult, InspectionResult, SearchAnalyticsResponse, SearchAnalyticsRow,
    ServiceAccountCredentials, SiteEntry, SitesListResponse, TokenResponse,
    UrlInspectionResult,
};
pub use healing::{
    run_background_healing, AppliedFix, HealingConfig, HealingLoop, HealingMode,
    HealingReport, IssueKind, SeoIssue, Severity,
};
pub use negotiation::{detect_crawler, negotiate, render_adaptive, ResponseFormat};
pub use ontology::{
    build_knowledge_graph, generate_jsonld, generate_jsonld_graph,
    infer_entity_type, Entity, EntityType, KnowledgeGraph, Relationship,
};
pub use photon::{
    build_photon_response, build_robots_txt, build_sitemap_xml,
    CrawlHints, PhotonEntity, PhotonRelationship, PhotonResponse,
};

/// Error type for AION.
#[derive(Debug, thiserror::Error)]
pub enum AionError {
    #[error("ontology error: {0}")]
    Ontology(String),
    #[error("negotiation error: {0}")]
    Negotiation(String),
    #[error("render error: {0}")]
    Render(String),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, AionError>;
