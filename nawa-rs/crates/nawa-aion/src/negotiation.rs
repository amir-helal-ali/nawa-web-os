//! Adaptive Negotiation Protocol — picks the best response format for each crawler.
//!
//! When a request arrives, AION inspects the User-Agent + Accept headers and
//! negotiates the optimal format:
//!
//! - Googlebot → HTML + JSON-LD
//! - GPTBot / ChatGPT / Anthropic → Markdown + JSON-LD (cleaner for AI training)
//! - Facebookbot / LinkedInbot → HTML + Open Graph
//! - Twitterbot / Telegrambot → HTML + Twitter Cards
//! - RSS readers → RSS XML
//! - Atom readers → Atom XML
//! - JSON API clients → pure JSON
//! - Default (users) → HTML with hydration
//!
//! This is NOT cloaking — the underlying content is identical, only the format
//! representation differs. Google explicitly allows format negotiation:
//! https://developers.google.com/search/docs/advanced/javascript/dynamic-rendering

use crate::ontology::{Entity, EntityType, KnowledgeGraph};
use serde::{Deserialize, Serialize};

/// The response format chosen by the negotiation protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResponseFormat {
    /// HTML + JSON-LD structured data (default for search engines).
    HtmlWithJsonLd,
    /// Markdown + JSON-LD comment (for AI crawlers like GPTBot).
    MarkdownWithJsonLd,
    /// HTML + Open Graph meta tags (Facebook, LinkedIn).
    HtmlWithOpenGraph,
    /// HTML + Twitter Card meta tags.
    HtmlWithTwitterCards,
    /// RSS 2.0 XML feed.
    Rss,
    /// Atom 1.0 XML feed.
    Atom,
    /// Pure JSON-LD (for structured-data consumers).
    JsonLd,
    /// Pure JSON (for API clients).
    Json,
    /// Pure Markdown (no metadata).
    Markdown,
    /// HTML with hydration script (for regular users — SvelteKit mode).
    HtmlWithHydration,
}

impl ResponseFormat {
    /// The Content-Type header for this format.
    pub fn content_type(&self) -> &'static str {
        match self {
            ResponseFormat::HtmlWithJsonLd
            | ResponseFormat::HtmlWithOpenGraph
            | ResponseFormat::HtmlWithTwitterCards
            | ResponseFormat::HtmlWithHydration => "text/html; charset=utf-8",
            ResponseFormat::MarkdownWithJsonLd
            | ResponseFormat::Markdown => "text/markdown; charset=utf-8",
            ResponseFormat::Rss => "application/rss+xml; charset=utf-8",
            ResponseFormat::Atom => "application/atom+xml; charset=utf-8",
            ResponseFormat::JsonLd => "application/ld+json; charset=utf-8",
            ResponseFormat::Json => "application/json; charset=utf-8",
        }
    }
}

/// Negotiate the best response format based on the request.
///
/// Examines (in order):
/// 1. User-Agent (crawler-specific optimization)
/// 2. Accept header (content negotiation)
/// 3. Default (HTML with hydration for users)
pub fn negotiate(user_agent: &str, accept: &str) -> ResponseFormat {
    let ua = user_agent.to_lowercase();

    // ── Layer 1: User-Agent-based crawler detection ──
    if ua.contains("gptbot") || ua.contains("chatgpt-user") || ua.contains("anthropic")
        || ua.contains("claudebot") || ua.contains("perplexitybot") {
        return ResponseFormat::MarkdownWithJsonLd;
    }
    if ua.contains("googlebot") || ua.contains("bingbot") || ua.contains("slurp")
        || ua.contains("duckduckbot") || ua.contains("baiduspider") || ua.contains("yandexbot")
        || ua.contains("applebot") || ua.contains("petalbot") {
        return ResponseFormat::HtmlWithJsonLd;
    }
    if ua.contains("facebookexternalhit") || ua.contains("linkedinbot")
        || ua.contains("whatsapp") || ua.contains("pinterest") {
        return ResponseFormat::HtmlWithOpenGraph;
    }
    if ua.contains("twitterbot") || ua.contains("telegrambot") {
        return ResponseFormat::HtmlWithTwitterCards;
    }
    if ua.contains("rss") || ua.contains("feedburner") || ua.contains("feedly") {
        return ResponseFormat::Rss;
    }

    // ── Layer 2: Accept-header negotiation ──
    let acc = accept.to_lowercase();
    if acc.contains("application/ld+json") {
        return ResponseFormat::JsonLd;
    }
    if acc.contains("text/markdown") {
        return ResponseFormat::Markdown;
    }
    if acc.contains("application/rss") {
        return ResponseFormat::Rss;
    }
    if acc.contains("application/atom") {
        return ResponseFormat::Atom;
    }
    if acc.contains("application/json") && !acc.contains("text/html") {
        return ResponseFormat::Json;
    }

    // ── Layer 3: Default — HTML for users ──
    ResponseFormat::HtmlWithHydration
}

/// Detect the crawler name (for analytics / logging).
pub fn detect_crawler(user_agent: &str) -> &'static str {
    let ua = user_agent.to_lowercase();
    if ua.contains("gptbot") { "gptbot" }
    else if ua.contains("chatgpt") { "chatgpt" }
    else if ua.contains("claude") { "claude" }
    else if ua.contains("anthropic") { "anthropic" }
    else if ua.contains("perplexity") { "perplexity" }
    else if ua.contains("googlebot") { "googlebot" }
    else if ua.contains("bingbot") { "bingbot" }
    else if ua.contains("slurp") { "yahoo" }
    else if ua.contains("duckduckbot") { "duckduckgo" }
    else if ua.contains("baiduspider") { "baidu" }
    else if ua.contains("yandexbot") { "yandex" }
    else if ua.contains("facebookexternalhit") { "facebook" }
    else if ua.contains("linkedinbot") { "linkedin" }
    else if ua.contains("twitterbot") { "twitter" }
    else if ua.contains("telegrambot") { "telegram" }
    else if ua.contains("whatsapp") { "whatsapp" }
    else if ua.contains("applebot") { "apple" }
    else if ua.contains("petalbot") { "huawei" }
    else { "user" }
}

/// Render an entity in the chosen format.
pub fn render_adaptive(
    format: ResponseFormat,
    entity: &Entity,
    graph: &KnowledgeGraph,
    ws_url: &str,
    site_url: &str,
) -> Vec<u8> {
    match format {
        ResponseFormat::HtmlWithJsonLd => render_html_with_jsonld(entity, graph, ws_url).into_bytes(),
        ResponseFormat::MarkdownWithJsonLd => render_markdown_with_jsonld(entity, graph).into_bytes(),
        ResponseFormat::HtmlWithOpenGraph => render_html_with_og(entity, graph, site_url).into_bytes(),
        ResponseFormat::HtmlWithTwitterCards => render_html_with_twitter(entity, graph, site_url).into_bytes(),
        ResponseFormat::Rss => render_rss(entity, graph, site_url).into_bytes(),
        ResponseFormat::Atom => render_atom(entity, graph, site_url).into_bytes(),
        ResponseFormat::JsonLd => crate::ontology::generate_jsonld(entity).to_string().into_bytes(),
        ResponseFormat::Json => entity.properties.to_string().into_bytes(),
        ResponseFormat::Markdown => render_markdown_only(entity).into_bytes(),
        ResponseFormat::HtmlWithHydration => render_html_full(entity, graph, ws_url).into_bytes(),
    }
}

/// HTML + JSON-LD (for Googlebot, Bingbot).
fn render_html_with_jsonld(entity: &Entity, _graph: &KnowledgeGraph, _ws_url: &str) -> String {
    let title = get_field(entity, "title")
        .or_else(|| get_field(entity, "headline"))
        .or_else(|| get_field(entity, "name"))
        .unwrap_or_else(|| entity.id.clone());
    let description = get_field(entity, "description")
        .or_else(|| get_field(entity, "bio"))
        .or_else(|| get_field(entity, "summary"))
        .unwrap_or_default();
    let jsonld = crate::ontology::generate_jsonld(entity);

    format!(r#"<!DOCTYPE html>
<html lang="ar" dir="rtl">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width,initial-scale=1.0">
<title>{title}</title>
<meta name="description" content="{description}">
<link rel="canonical" href="{url}">
<script type="application/ld+json">
{jsonld}
</script>
</head>
<body>
<article>
<h1>{title}</h1>
<p>{description}</p>
</article>
</body>
</html>"#,
        title = escape_html(&title),
        description = escape_html(&description),
        url = entity.url,
        jsonld = serde_json::to_string_pretty(&jsonld).unwrap_or_default()
    )
}

/// HTML + Open Graph (for Facebook, LinkedIn, WhatsApp).
fn render_html_with_og(entity: &Entity, _graph: &KnowledgeGraph, site_url: &str) -> String {
    let title = get_field(entity, "title")
        .or_else(|| get_field(entity, "headline"))
        .or_else(|| get_field(entity, "name"))
        .unwrap_or_else(|| entity.id.clone());
    let description = get_field(entity, "description").unwrap_or_default();
    let image = get_field(entity, "image")
        .or_else(|| get_field(entity, "og_image"))
        .map(|s| format!("{site_url}{s}"))
        .unwrap_or_else(|| format!("{site_url}/api/og/{}.png", entity.id.replace('/', "_")));
    let url = format!("{}{}", site_url, entity.url);

    format!(r#"<!DOCTYPE html>
<html lang="ar" dir="rtl">
<head>
<meta charset="UTF-8">
<title>{title}</title>
<meta property="og:title" content="{title}">
<meta property="og:description" content="{description}">
<meta property="og:image" content="{image}">
<meta property="og:url" content="{url}">
<meta property="og:type" content="{og_type}">
<meta property="og:site_name" content="NAWA">
</head>
<body>
<article>
<h1>{title}</h1>
<p>{description}</p>
</article>
</body>
</html>"#,
        title = escape_html(&title),
        description = escape_html(&description),
        image = escape_html(&image),
        url = escape_html(&url),
        og_type = og_type_for_entity(entity.entity_type)
    )
}

/// HTML + Twitter Cards.
fn render_html_with_twitter(entity: &Entity, _graph: &KnowledgeGraph, site_url: &str) -> String {
    let title = get_field(entity, "title")
        .or_else(|| get_field(entity, "name"))
        .unwrap_or_else(|| entity.id.clone());
    let description = get_field(entity, "description").unwrap_or_default();
    let image = get_field(entity, "image")
        .map(|s| format!("{site_url}{s}"))
        .unwrap_or_else(|| format!("{site_url}/api/og/{}.png", entity.id.replace('/', "_")));

    format!(r#"<!DOCTYPE html>
<html lang="ar" dir="rtl">
<head>
<meta charset="UTF-8">
<title>{title}</title>
<meta name="twitter:card" content="summary_large_image">
<meta name="twitter:title" content="{title}">
<meta name="twitter:description" content="{description}">
<meta name="twitter:image" content="{image}">
</head>
<body>
<article>
<h1>{title}</h1>
<p>{description}</p>
</article>
</body>
</html>"#,
        title = escape_html(&title),
        description = escape_html(&description),
        image = escape_html(&image)
    )
}

/// Markdown + JSON-LD comment (for GPTBot, Claude, Perplexity).
fn render_markdown_with_jsonld(entity: &Entity, _graph: &KnowledgeGraph) -> String {
    let title = get_field(entity, "title")
        .or_else(|| get_field(entity, "headline"))
        .or_else(|| get_field(entity, "name"))
        .unwrap_or_else(|| entity.id.clone());
    let description = get_field(entity, "description").unwrap_or_default();
    let jsonld = crate::ontology::generate_jsonld(entity);

    let mut md = format!("# {}\n\n{}\n\n", title, description);

    // Add structured data as an HTML comment (AI crawlers see it, humans don't).
    md.push_str(&format!("<!-- JSON-LD: {} -->\n", serde_json::to_string(&jsonld).unwrap_or_default()));

    // Add all properties as a definition list.
    if let Some(props) = entity.properties.as_object() {
        md.push_str("\n## Properties\n\n");
        for (k, v) in props {
            if k == "password_hash" || k == "password" || k == "token" { continue; }
            md.push_str(&format!("- **{}**: {}\n", k, v));
        }
    }

    md
}

/// Pure Markdown (no metadata).
fn render_markdown_only(entity: &Entity) -> String {
    let title = get_field(entity, "title")
        .or_else(|| get_field(entity, "name"))
        .unwrap_or_else(|| entity.id.clone());
    let description = get_field(entity, "description").unwrap_or_default();

    let mut md = format!("# {}\n\n{}\n", title, description);

    if let Some(props) = entity.properties.as_object() {
        md.push_str("\n## Properties\n\n");
        for (k, v) in props {
            if k == "password_hash" || k == "password" || k == "token" { continue; }
            md.push_str(&format!("- **{}**: {}\n", k, v));
        }
    }

    md
}

/// RSS 2.0 XML (for feed readers).
fn render_rss(entity: &Entity, _graph: &KnowledgeGraph, site_url: &str) -> String {
    let title = get_field(entity, "title").unwrap_or_else(|| entity.id.clone());
    let description = get_field(entity, "description").unwrap_or_default();
    let pub_date = get_field(entity, "date_published")
        .or_else(|| get_field(entity, "published_at"))
        .unwrap_or_else(|| entity.last_modified.clone());

    format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
<channel>
<title>NAWA — {title}</title>
<link>{site_url}{url}</link>
<description>{description}</description>
<language>ar</language>
<item>
<title>{title}</title>
<link>{site_url}{url}</link>
<description>{description}</description>
<pubDate>{pub_date}</pubDate>
<guid>{site_url}{url}</guid>
</item>
</channel>
</rss>"#,
        title = escape_xml(&title),
        site_url = site_url,
        url = entity.url,
        description = escape_xml(&description),
        pub_date = escape_xml(&pub_date)
    )
}

/// Atom 1.0 XML.
fn render_atom(entity: &Entity, _graph: &KnowledgeGraph, site_url: &str) -> String {
    let title = get_field(entity, "title").unwrap_or_else(|| entity.id.clone());
    let description = get_field(entity, "description").unwrap_or_default();
    let updated = entity.last_modified.clone();

    format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
<title>{title}</title>
<link href="{site_url}{url}"/>
<updated>{updated}</updated>
<id>{site_url}{url}</id>
<entry>
<title>{title}</title>
<link href="{site_url}{url}"/>
<id>{site_url}{url}</id>
<updated>{updated}</updated>
<summary>{description}</summary>
</entry>
</feed>"#,
        title = escape_xml(&title),
        site_url = site_url,
        url = entity.url,
        updated = escape_xml(&updated),
        description = escape_xml(&description)
    )
}

/// HTML with hydration script (for regular users — SPA mode).
fn render_html_full(entity: &Entity, _graph: &KnowledgeGraph, ws_url: &str) -> String {
    let title = get_field(entity, "title")
        .or_else(|| get_field(entity, "name"))
        .unwrap_or_else(|| entity.id.clone());

    format!(r#"<!DOCTYPE html>
<html lang="ar" dir="rtl">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width,initial-scale=1.0">
<title>{title}</title>
<script>window.__NAWA__ = {{ wsUrl: "{ws_url}", polling: false }};</script>
</head>
<body>
<div id="svelte">Loading…</div>
</body>
</html>"#,
        title = escape_html(&title),
        ws_url = ws_url
    )
}

// ── Helpers ──

fn get_field(entity: &Entity, field: &str) -> Option<String> {
    entity.properties.get(field)
        .and_then(|v| {
            if v.is_string() { v.as_str().map(|s| s.to_string()) }
            else { Some(v.to_string()) }
        })
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
     .replace('<', "&lt;")
     .replace('>', "&gt;")
     .replace('"', "&quot;")
     .replace('\'', "&#39;")
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
     .replace('<', "&lt;")
     .replace('>', "&gt;")
     .replace('"', "&quot;")
     .replace('\'', "&apos;")
}

fn og_type_for_entity(et: EntityType) -> &'static str {
    match et {
        EntityType::Person => "profile",
        EntityType::Article => "article",
        EntityType::Product => "product",
        EntityType::Event => "event",
        EntityType::Organization => "website",
        _ => "website",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_entity() -> Entity {
        Entity {
            id: "posts/hello-nawa".into(),
            entity_type: EntityType::Article,
            url: "/posts/hello-nawa".into(),
            properties: serde_json::json!({
                "title": "Hello NAWA",
                "description": "First post",
                "author": "42",
                "date_published": "2026-01-01"
            }),
            last_modified: "2026-01-01".into(),
            importance: 0.9,
        }
    }

    fn sample_graph() -> KnowledgeGraph {
        let mut g = KnowledgeGraph::new();
        g.add_entity(sample_entity());
        g
    }

    #[test]
    fn googlebot_gets_html_with_jsonld() {
        let fmt = negotiate("Mozilla/5.0 (compatible; Googlebot/2.1)", "text/html");
        assert_eq!(fmt, ResponseFormat::HtmlWithJsonLd);
    }

    #[test]
    fn gptbot_gets_markdown() {
        let fmt = negotiate("GPTBot/1.0", "*/*");
        assert_eq!(fmt, ResponseFormat::MarkdownWithJsonLd);
    }

    #[test]
    fn facebookbot_gets_open_graph() {
        let fmt = negotiate("facebookexternalhit/1.1", "text/html");
        assert_eq!(fmt, ResponseFormat::HtmlWithOpenGraph);
    }

    #[test]
    fn twitterbot_gets_twitter_cards() {
        let fmt = negotiate("Twitterbot/1.0", "text/html");
        assert_eq!(fmt, ResponseFormat::HtmlWithTwitterCards);
    }

    #[test]
    fn default_user_gets_html_with_hydration() {
        let fmt = negotiate("Mozilla/5.0 (X11; Linux x86_64) Chrome/120", "text/html");
        assert_eq!(fmt, ResponseFormat::HtmlWithHydration);
    }

    #[test]
    fn json_accept_returns_json() {
        let fmt = negotiate("curl/7.0", "application/json");
        assert_eq!(fmt, ResponseFormat::Json);
    }

    #[test]
    fn detect_crawler_identifies_googlebot() {
        assert_eq!(detect_crawler("Googlebot/2.1"), "googlebot");
        assert_eq!(detect_crawler("GPTBot/1.0"), "gptbot");
        assert_eq!(detect_crawler("Mozilla/5.0 Firefox"), "user");
    }

    #[test]
    fn html_with_jsonld_contains_structured_data() {
        let entity = sample_entity();
        let graph = sample_graph();
        let html = render_html_with_jsonld(&entity, &graph, "ws://localhost:8081");
        assert!(html.contains("application/ld+json"));
        assert!(html.contains("Article"));
        assert!(html.contains("Hello NAWA"));
        assert!(html.contains("canonical"));
    }

    #[test]
    fn html_with_og_contains_open_graph_tags() {
        let entity = sample_entity();
        let graph = sample_graph();
        let html = render_html_with_og(&entity, &graph, "https://example.com");
        assert!(html.contains("og:title"));
        assert!(html.contains("og:image"));
        assert!(html.contains("og:type"));
    }

    #[test]
    fn markdown_with_jsonld_has_html_comment() {
        let entity = sample_entity();
        let graph = sample_graph();
        let md = render_markdown_with_jsonld(&entity, &graph);
        assert!(md.starts_with("# Hello NAWA"));
        assert!(md.contains("JSON-LD"));
    }

    #[test]
    fn rss_is_valid_xml() {
        let entity = sample_entity();
        let graph = sample_graph();
        let rss = render_rss(&entity, &graph, "https://example.com");
        assert!(rss.contains("<?xml"));
        assert!(rss.contains("<rss"));
        assert!(rss.contains("<item>"));
    }

    #[test]
    fn atom_is_valid_xml() {
        let entity = sample_entity();
        let graph = sample_graph();
        let atom = render_atom(&entity, &graph, "https://example.com");
        assert!(atom.contains("<?xml"));
        assert!(atom.contains("<feed"));
        assert!(atom.contains("<entry>"));
    }

    #[test]
    fn escape_html_handles_special_chars() {
        assert_eq!(escape_html("a < b > c & d"), "a &lt; b &gt; c &amp; d");
    }

    #[test]
    fn og_type_maps_correctly() {
        assert_eq!(og_type_for_entity(EntityType::Person), "profile");
        assert_eq!(og_type_for_entity(EntityType::Article), "article");
        assert_eq!(og_type_for_entity(EntityType::Product), "product");
        assert_eq!(og_type_for_entity(EntityType::WebPage), "website");
    }

    #[test]
    fn content_type_is_correct() {
        assert_eq!(ResponseFormat::HtmlWithJsonLd.content_type(), "text/html; charset=utf-8");
        assert_eq!(ResponseFormat::Markdown.content_type(), "text/markdown; charset=utf-8");
        assert_eq!(ResponseFormat::Rss.content_type(), "application/rss+xml; charset=utf-8");
        assert_eq!(ResponseFormat::JsonLd.content_type(), "application/ld+json; charset=utf-8");
    }
}
