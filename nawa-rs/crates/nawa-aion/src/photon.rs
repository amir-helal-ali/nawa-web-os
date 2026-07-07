//! Photon Index Protocol (PIP) — single endpoint that exposes the entire Knowledge Graph.
//!
//! Instead of forcing crawlers to request thousands of pages, PIP exposes
//! `/__photon__` — a single endpoint that returns:
//! - The complete Knowledge Graph (entities + relationships)
//! - Crawl hints (priority URLs, URLs to avoid)
//! - Predictions (likely new URLs)
//! - Format availability (which response formats each entity supports)
//!
//! This is similar to Google's "Sitemaps" but far richer:
//! - Sitemap: flat list of URLs
//! - PIP: rich graph with relationships, formats, and intelligence

use crate::ontology::{EntityType, KnowledgeGraph};
use serde::{Deserialize, Serialize};

/// The PIP response — returned by `GET /__photon__`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhotonResponse {
    /// Protocol version.
    pub protocol: String,
    /// When this response was generated (ISO 8601).
    pub generated_at: String,
    /// Site URL (e.g., "https://example.com").
    pub site_url: String,
    /// Total entity count.
    pub entity_count: usize,
    /// Total relationship count.
    pub relationship_count: usize,
    /// All entities (compact representation).
    pub entities: Vec<PhotonEntity>,
    /// All relationships.
    pub relationships: Vec<PhotonRelationship>,
    /// Crawl hints — what to prioritize and what to avoid.
    pub crawl_hints: CrawlHints,
    /// Predicted high-priority URLs (crawlers should fetch these first).
    pub priority_urls: Vec<String>,
    /// Supported response formats.
    pub supported_formats: Vec<String>,
}

/// Compact entity representation for PIP.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhotonEntity {
    pub id: String,
    pub entity_type: String,
    pub url: String,
    pub last_modified: String,
    pub etag: String,
    pub importance: f32,
    pub change_frequency: String,
    pub formats: Vec<String>,
    pub title: Option<String>,
    pub description: Option<String>,
}

/// Compact relationship representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhotonRelationship {
    pub from: String,
    pub to: String,
    pub rel_type: String,
}

/// Crawl hints — guide crawlers to optimal behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlHints {
    /// URLs that change frequently (poll more often).
    pub high_change_urls: Vec<String>,
    /// URLs that are stable (poll less often).
    pub stable_urls: Vec<String>,
    /// URLs to avoid (admin, API, etc.).
    pub avoid_urls: Vec<String>,
    /// Suggested crawl rate (requests/second).
    pub rate_limit: u32,
}

/// Build the PIP response from a Knowledge Graph.
pub fn build_photon_response(
    graph: &KnowledgeGraph,
    site_url: &str,
) -> PhotonResponse {
    let entities: Vec<PhotonEntity> = graph.entities.iter()
        .map(|e| PhotonEntity {
            id: e.id.clone(),
            entity_type: e.entity_type.short_name().to_string(),
            url: e.url.clone(),
            last_modified: e.last_modified.clone(),
            etag: compute_etag(&e.id, &e.last_modified),
            importance: e.importance,
            change_frequency: change_freq_for(e.entity_type).to_string(),
            formats: supported_formats_for(e.entity_type),
            title: get_field(&e.properties, "title")
                .or_else(|| get_field(&e.properties, "headline"))
                .or_else(|| get_field(&e.properties, "name")),
            description: get_field(&e.properties, "description"),
        })
        .collect();

    let relationships: Vec<PhotonRelationship> = graph.relationships.iter()
        .map(|r| PhotonRelationship {
            from: r.from.clone(),
            to: r.to.clone(),
            rel_type: r.rel_type.clone(),
        })
        .collect();

    // Build crawl hints.
    let mut high_change = Vec::new();
    let mut stable = Vec::new();
    let avoid = vec![
        "/admin".to_string(), "/api".to_string(), "/settings".to_string(),
        "/profile".to_string(), "/backup".to_string(), "/restore".to_string(),
    ];

    for e in &graph.entities {
        match e.entity_type {
            EntityType::Article | EntityType::Event => high_change.push(e.url.clone()),
            EntityType::Person | EntityType::Organization => stable.push(e.url.clone()),
            _ => stable.push(e.url.clone()),
        }
    }

    // Priority URLs: high-importance entities first.
    let mut priority: Vec<String> = graph.entities.iter()
        .filter(|e| e.importance >= 0.7)
        .map(|e| e.url.clone())
        .collect();
    priority.sort_by(|a, b| b.len().cmp(&a.len()));  // shorter URLs first (more general)

    PhotonResponse {
        protocol: "photon/1.0".to_string(),
        generated_at: chrono::Utc::now().to_rfc3339(),
        site_url: site_url.to_string(),
        entity_count: graph.entity_count(),
        relationship_count: graph.relationship_count(),
        entities,
        relationships,
        crawl_hints: CrawlHints {
            high_change_urls: high_change,
            stable_urls: stable,
            avoid_urls: avoid,
            rate_limit: 10,
        },
        priority_urls: priority,
        supported_formats: vec![
            "html+jsonld".into(), "markdown+jsonld".into(), "html+og".into(),
            "html+twitter".into(), "rss".into(), "atom".into(),
            "jsonld".into(), "json".into(), "markdown".into(),
        ],
    }
}

/// Generate a sitemap.xml from the Knowledge Graph.
pub fn build_sitemap_xml(graph: &KnowledgeGraph, site_url: &str) -> String {
    let mut xml = String::from(r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">"#);

    for e in &graph.entities {
        xml.push_str(&format!(r#"
  <url>
    <loc>{site_url}{url}</loc>
    <lastmod>{lastmod}</lastmod>
    <changefreq>{freq}</changefreq>
    <priority>{priority:.1}</priority>
  </url>"#,
            site_url = site_url,
            url = e.url,
            lastmod = e.last_modified,
            freq = change_freq_for(e.entity_type),
            priority = e.importance
        ));
    }

    xml.push_str("\n</urlset>");
    xml
}

/// Generate robots.txt (dynamic — adapts to known entity types).
pub fn build_robots_txt(site_url: &str) -> String {
    format!(r#"User-agent: *
Allow: /
Disallow: /admin
Disallow: /api
Disallow: /settings
Disallow: /profile
Disallow: /backup
Disallow: /restore
Disallow: /password-reset

# AI crawlers welcome (NAWA serves Markdown for them)
User-agent: GPTBot
Allow: /

User-agent: ChatGPT-User
Allow: /

User-agent: ClaudeBot
Allow: /

User-agent: PerplexityBot
Allow: /

Crawl-delay: 0.5

Sitemap: {site_url}/sitemap.xml
"#, site_url = site_url)
}

// ── Helpers ──

fn compute_etag(id: &str, last_modified: &str) -> String {
    use xxhash_rust::xxh3::xxh3_64;
    let combined = format!("{id}|{last_modified}");
    format!("\"{:x}\"", xxh3_64(combined.as_bytes()))
}

fn change_freq_for(et: EntityType) -> &'static str {
    match et {
        EntityType::Article => "weekly",
        EntityType::Product => "daily",
        EntityType::Event => "daily",
        EntityType::Person => "monthly",
        EntityType::Organization => "monthly",
        EntityType::Review => "weekly",
        EntityType::Recipe => "monthly",
        EntityType::FAQPage => "monthly",
        EntityType::WebPage => "weekly",
    }
}

fn supported_formats_for(et: EntityType) -> Vec<String> {
    match et {
        EntityType::Article => vec!["html+jsonld".into(), "markdown+jsonld".into(), "rss".into(), "atom".into()],
        EntityType::Product => vec!["html+jsonld".into(), "json".into()],
        EntityType::Person => vec!["html+jsonld".into(), "html+og".into()],
        EntityType::Event => vec!["html+jsonld".into(), "rss".into()],
        _ => vec!["html+jsonld".into()],
    }
}

fn get_field(props: &serde_json::Value, field: &str) -> Option<String> {
    props.get(field)
        .and_then(|v| {
            if v.is_string() { v.as_str().map(|s| s.to_string()) }
            else { Some(v.to_string()) }
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ontology::{Entity, KnowledgeGraph};

    fn sample_graph() -> KnowledgeGraph {
        let mut g = KnowledgeGraph::new();
        g.add_entity(Entity {
            id: "posts/hello".into(),
            entity_type: EntityType::Article,
            url: "/posts/hello".into(),
            properties: serde_json::json!({"title": "Hello", "description": "First"}),
            last_modified: "2026-01-01T00:00:00Z".into(),
            importance: 0.9,
        });
        g.add_entity(Entity {
            id: "users/42".into(),
            entity_type: EntityType::Person,
            url: "/users/42".into(),
            properties: serde_json::json!({"name": "Admin"}),
            last_modified: "2026-01-01T00:00:00Z".into(),
            importance: 0.7,
        });
        g.add_relationship("posts/hello", "authoredBy", "users/42");
        g
    }

    #[test]
    fn photon_response_has_protocol_version() {
        let g = sample_graph();
        let r = build_photon_response(&g, "https://example.com");
        assert_eq!(r.protocol, "photon/1.0");
        assert_eq!(r.entity_count, 2);
        assert_eq!(r.relationship_count, 1);
    }

    #[test]
    fn photon_includes_crawl_hints() {
        let g = sample_graph();
        let r = build_photon_response(&g, "https://example.com");
        assert!(!r.crawl_hints.avoid_urls.is_empty());
        assert!(r.crawl_hints.avoid_urls.contains(&"/admin".to_string()));
        assert!(r.crawl_hints.rate_limit > 0);
    }

    #[test]
    fn photon_priority_urls_filters_by_importance() {
        let g = sample_graph();
        let r = build_photon_response(&g, "https://example.com");
        // Both entities have importance >= 0.7
        assert_eq!(r.priority_urls.len(), 2);
    }

    #[test]
    fn photon_entities_have_etags() {
        let g = sample_graph();
        let r = build_photon_response(&g, "https://example.com");
        for e in &r.entities {
            assert!(e.etag.starts_with('"'));
            assert!(e.etag.ends_with('"'));
        }
    }

    #[test]
    fn sitemap_xml_is_valid() {
        let g = sample_graph();
        let xml = build_sitemap_xml(&g, "https://example.com");
        assert!(xml.contains("<?xml"));
        assert!(xml.contains("<urlset"));
        assert!(xml.contains("<loc>https://example.com/posts/hello</loc>"));
        assert!(xml.contains("<priority>"));
    }

    #[test]
    fn robots_txt_allows_ai_crawlers() {
        let txt = build_robots_txt("https://example.com");
        assert!(txt.contains("GPTBot"));
        assert!(txt.contains("ClaudeBot"));
        assert!(txt.contains("Sitemap: https://example.com/sitemap.xml"));
        assert!(txt.contains("Disallow: /admin"));
    }

    #[test]
    fn change_freq_is_sensible() {
        assert_eq!(change_freq_for(EntityType::Article), "weekly");
        assert_eq!(change_freq_for(EntityType::Product), "daily");
        assert_eq!(change_freq_for(EntityType::Person), "monthly");
    }

    #[test]
    fn supported_formats_match_entity_type() {
        let article_formats = supported_formats_for(EntityType::Article);
        assert!(article_formats.contains(&"markdown+jsonld".to_string()));
        assert!(article_formats.contains(&"rss".to_string()));

        let person_formats = supported_formats_for(EntityType::Person);
        assert!(person_formats.contains(&"html+og".to_string()));
    }

    #[test]
    fn etag_is_deterministic() {
        let e1 = compute_etag("posts/hello", "2026-01-01");
        let e2 = compute_etag("posts/hello", "2026-01-01");
        assert_eq!(e1, e2);

        let e3 = compute_etag("posts/hello", "2026-01-02");
        assert_ne!(e1, e3);
    }

    #[test]
    fn photon_serializes_to_json() {
        let g = sample_graph();
        let r = build_photon_response(&g, "https://example.com");
        let json = serde_json::to_string(&r).unwrap();
        assert!(json.contains("photon/1.0"));
        assert!(json.contains("posts/hello"));
    }
}
