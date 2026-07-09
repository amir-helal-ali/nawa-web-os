//! Ontological Inference Engine — infers entity types from DB structure.
//!
//! Instead of requiring manual schema declarations, AION analyzes the structure
//! of each DB record and infers its ontological type (Person, Article, Product,
//! Event, etc.) using a decision tree based on field presence.
//!
//! This builds a Knowledge Graph automatically — Google can index entities, not
//! just pages, enabling Google Discover and rich results.

use serde::{Deserialize, Serialize};
// HashMap not currently used directly but kept for future extensions.
#[allow(unused_imports)]
use std::collections::HashMap;

/// An entity type — corresponds to schema.org types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EntityType {
    Person,
    Article,
    Product,
    Event,
    Organization,
    Review,
    Recipe,
    FAQPage,
    WebPage,
}

impl EntityType {
    /// The schema.org type URL.
    pub fn schema_url(&self) -> &'static str {
        match self {
            EntityType::Person => "https://schema.org/Person",
            EntityType::Article => "https://schema.org/Article",
            EntityType::Product => "https://schema.org/Product",
            EntityType::Event => "https://schema.org/Event",
            EntityType::Organization => "https://schema.org/Organization",
            EntityType::Review => "https://schema.org/Review",
            EntityType::Recipe => "https://schema.org/Recipe",
            EntityType::FAQPage => "https://schema.org/FAQPage",
            EntityType::WebPage => "https://schema.org/WebPage",
        }
    }

    /// Short type name (for JSON-LD "@type").
    pub fn short_name(&self) -> &'static str {
        match self {
            EntityType::Person => "Person",
            EntityType::Article => "Article",
            EntityType::Product => "Product",
            EntityType::Event => "Event",
            EntityType::Organization => "Organization",
            EntityType::Review => "Review",
            EntityType::Recipe => "Recipe",
            EntityType::FAQPage => "FAQPage",
            EntityType::WebPage => "WebPage",
        }
    }
}

/// An entity — a single node in the Knowledge Graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    /// Unique ID (e.g., "/users/42", "/posts/hello-nawa").
    pub id: String,
    /// Inferred entity type.
    pub entity_type: EntityType,
    /// URL the entity is reachable at.
    pub url: String,
    /// Properties (JSON object — from DB record).
    pub properties: serde_json::Value,
    /// Last-modified timestamp (ISO 8601).
    pub last_modified: String,
    /// Importance score (0.0–1.0) — used for sitemap priority.
    pub importance: f32,
}

/// A relationship between two entities (edge in Knowledge Graph).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub from: String,
    pub to: String,
    /// Relationship type (e.g., "authoredBy", "partOf", "review", "mentions").
    pub rel_type: String,
}

/// The Knowledge Graph — entities + relationships.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeGraph {
    pub entities: Vec<Entity>,
    pub relationships: Vec<Relationship>,
    pub generated_at: String,
}

impl Default for KnowledgeGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl KnowledgeGraph {
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
            relationships: Vec::new(),
            generated_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    pub fn add_entity(&mut self, entity: Entity) {
        self.entities.push(entity);
    }

    pub fn add_relationship(&mut self, from: &str, rel_type: &str, to: &str) {
        self.relationships.push(Relationship {
            from: from.to_string(),
            to: to.to_string(),
            rel_type: rel_type.to_string(),
        });
    }

    /// Find an entity by ID.
    pub fn find(&self, id: &str) -> Option<&Entity> {
        self.entities.iter().find(|e| e.id == id)
    }

    /// Total entity count.
    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    /// Total relationship count.
    pub fn relationship_count(&self) -> usize {
        self.relationships.len()
    }
}

/// Infer an entity's type from its key + properties.
///
/// Uses a decision tree based on field presence (similar to data-science
/// heuristics). Falls back to key prefix if no signals match.
pub fn infer_entity_type(key: &str, data: &serde_json::Value) -> EntityType {
    let obj = match data.as_object() {
        Some(o) => o,
        None => return EntityType::WebPage,
    };

    // Decision tree — most specific first.
    let has = |field: &str| -> bool { obj.contains_key(field) };

    // Person: has identity + auth fields.
    if (has("email") || has("username")) && (has("password_hash") || has("name")) {
        return EntityType::Person;
    }
    // Article: has headline + author + date.
    if (has("headline") || has("title")) && has("author") && (has("date_published") || has("published_at")) {
        return EntityType::Article;
    }
    // Product: has price + image + availability.
    if has("price") && (has("image") || has("availability") || has("sku")) {
        return EntityType::Product;
    }
    // Event: has start date + location.
    if (has("start_date") || has("startDate")) && (has("location") || has("venue")) {
        return EntityType::Event;
    }
    // Review: has reviewRating + reviewed item.
    if has("review_rating") || has("reviewRating") || (has("rating") && has("item_reviewed")) {
        return EntityType::Review;
    }
    // Recipe: has ingredients + instructions.
    if has("ingredients") && (has("instructions") || has("recipe_instructions")) {
        return EntityType::Recipe;
    }
    // Organization: has logo + address + same-site url.
    if has("logo") && (has("address") || has("founding_date")) {
        return EntityType::Organization;
    }
    // FAQPage: has questions array.
    if has("questions") || has("main_entity") {
        return EntityType::FAQPage;
    }

    // Fallback: infer from key prefix.
    match key.split(':').next() {
        Some("user") | Some("users") | Some("profile") => EntityType::Person,
        Some("post") | Some("posts") | Some("article") | Some("blog") => EntityType::Article,
        Some("product") | Some("products") => EntityType::Product,
        Some("event") | Some("events") => EntityType::Event,
        Some("org") | Some("organization") => EntityType::Organization,
        Some("review") | Some("reviews") => EntityType::Review,
        Some("recipe") | Some("recipes") => EntityType::Recipe,
        _ => EntityType::WebPage,
    }
}

/// Build a Knowledge Graph from NAWA-DB by scanning all records.
///
/// Each DB record is analyzed:
/// 1. Its entity type is inferred (via `infer_entity_type`).
/// 2. Its URL is derived from the key (e.g., "user:42" → "/users/42").
/// 3. Its relationships are inferred from foreign-key-like fields
///    (author_id, parent_id, mentions, etc.).
pub fn build_knowledge_graph(db: &nawa_db::DbEngine) -> KnowledgeGraph {
    let mut graph = KnowledgeGraph::new();
    let now = chrono::Utc::now().to_rfc3339();

    // Scan all keys (limit to 100k for safety on huge DBs).
    let records = db.scan_prefix("", 100_000);

    for (key_bytes, value) in records {
        let key = String::from_utf8_lossy(&key_bytes).to_string();

        // Skip internal NAWA-DB keys (auth, seo, etc.).
        if key.starts_with("auth:") || key.starts_with("seo:")
            || key.starts_with("user:email:") || key.starts_with("session:")
            || key.starts_with("settings:") {
            continue;
        }

        // Extract JSON value from the nawa_db::Value enum.
        let data: serde_json::Value = match &value {
            nawa_db::Value::Json(v) => v.clone(),
            nawa_db::Value::Bytes(b) => {
                // Try to parse as JSON, fallback to a string value.
                serde_json::from_slice(b).unwrap_or_else(|_| {
                    serde_json::Value::String(String::from_utf8_lossy(b).to_string())
                })
            }
        };

        let entity_type = infer_entity_type(&key, &data);
        let entity_id = key_to_entity_id(&key);
        let url = entity_id_to_url(&entity_id);

        let entity = Entity {
            id: entity_id.clone(),
            entity_type,
            url,
            properties: data.clone(),
            last_modified: now.clone(),
            importance: compute_importance(entity_type, &data),
        };

        // Infer relationships from foreign-key-like fields.
        infer_relationships(&mut graph, &entity, &data);

        graph.add_entity(entity);
    }

    graph
}

/// Convert a DB key to an entity ID (e.g., "user:42" → "users/42").
fn key_to_entity_id(key: &str) -> String {
    let parts: Vec<&str> = key.splitn(2, ':').collect();
    if parts.len() == 2 {
        let prefix = match parts[0] {
            "user" => "users",
            "post" => "posts",
            "product" => "products",
            "event" => "events",
            "org" | "organization" => "organizations",
            "review" => "reviews",
            "recipe" => "recipes",
            other => other,
        };
        format!("{}/{}", prefix, parts[1])
    } else {
        key.to_string()
    }
}

/// Convert an entity ID to a URL path (e.g., "users/42" → "/users/42").
fn entity_id_to_url(entity_id: &str) -> String {
    format!("/{}", entity_id)
}

/// Infer relationships from foreign-key-like fields in the entity data.
fn infer_relationships(graph: &mut KnowledgeGraph, entity: &Entity, data: &serde_json::Value) {
    let obj = match data.as_object() {
        Some(o) => o,
        None => return,
    };

    // author_id → authoredBy
    if let Some(author_id) = obj.get("author_id").and_then(|v| v.as_str()) {
        let author_entity = format!("users/{}", author_id);
        graph.add_relationship(&entity.id, "authoredBy", &author_entity);
    }
    // author (if it's an ID)
    if let Some(author) = obj.get("author").and_then(|v| v.as_str()) {
        if !author.contains(' ') {  // likely an ID, not a name
            let author_entity = format!("users/{}", author);
            graph.add_relationship(&entity.id, "authoredBy", &author_entity);
        }
    }
    // parent_id → partOf
    if let Some(parent_id) = obj.get("parent_id").and_then(|v| v.as_str()) {
        graph.add_relationship(&entity.id, "partOf", parent_id);
    }
    // mentions (array of IDs)
    if let Some(mentions) = obj.get("mentions").and_then(|v| v.as_array()) {
        for m in mentions {
            if let Some(m_id) = m.as_str() {
                graph.add_relationship(&entity.id, "mentions", m_id);
            }
        }
    }
    // review_of → reviews
    if let Some(reviewed) = obj.get("item_reviewed").and_then(|v| v.as_str()) {
        graph.add_relationship(&entity.id, "reviews", reviewed);
    }
}

/// Compute importance score (0.0–1.0) for sitemap priority.
fn compute_importance(entity_type: EntityType, _data: &serde_json::Value) -> f32 {
    match entity_type {
        EntityType::Article => 0.9,
        EntityType::Product => 0.9,
        EntityType::Person => 0.7,
        EntityType::Event => 0.8,
        EntityType::Organization => 0.6,
        EntityType::Review => 0.5,
        EntityType::Recipe => 0.7,
        EntityType::FAQPage => 0.6,
        EntityType::WebPage => 0.5,
    }
}

/// Generate JSON-LD for a single entity (auto-typed).
pub fn generate_jsonld(entity: &Entity) -> serde_json::Value {
    let mut jsonld = serde_json::json!({
        "@context": "https://schema.org",
        "@type": entity.entity_type.short_name(),
        "@id": entity.id,
        "url": entity.url,
    });

    // Merge in entity properties (the data fields).
    if let Some(obj) = jsonld.as_object_mut() {
        if let Some(props) = entity.properties.as_object() {
            for (k, v) in props {
                // Skip sensitive/private fields — NEVER export to SEO/SSR.
                // These fields must never appear in JSON-LD, sitemap, or SSR output.
                if matches!(k.as_str(),
                    "password_hash" | "password" | "token" | "secret" |
                    "email" | "phone" | "address" | "ip_address" |
                    "session_id" | "refresh_token" | "api_key" |
                    "credit_card" | "ssn" | "private_key"
                ) {
                    continue;
                }
                obj.insert(k.clone(), v.clone());
            }
        }
    }

    jsonld
}

/// Generate the full Knowledge Graph as JSON-LD `@graph`.
pub fn generate_jsonld_graph(graph: &KnowledgeGraph) -> serde_json::Value {
    let entities: Vec<serde_json::Value> = graph.entities.iter()
        .map(generate_jsonld)
        .collect();

    serde_json::json!({
        "@context": "https://schema.org",
        "@graph": entities
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn infers_person_from_auth_fields() {
        let data = serde_json::json!({
            "username": "admin", "email": "a@b.com", "password_hash": "xxx"
        });
        assert_eq!(infer_entity_type("user:1", &data), EntityType::Person);
    }

    #[test]
    fn infers_article_from_headline_author_date() {
        let data = serde_json::json!({
            "headline": "Hello", "author": "42", "date_published": "2026-01-01"
        });
        assert_eq!(infer_entity_type("post:1", &data), EntityType::Article);
    }

    #[test]
    fn infers_product_from_price_image() {
        let data = serde_json::json!({
            "price": 99.99, "image": "/img.jpg", "availability": "in stock"
        });
        assert_eq!(infer_entity_type("product:1", &data), EntityType::Product);
    }

    #[test]
    fn infers_event_from_start_date_location() {
        let data = serde_json::json!({
            "start_date": "2026-12-01", "location": "Cairo"
        });
        assert_eq!(infer_entity_type("event:1", &data), EntityType::Event);
    }

    #[test]
    fn infers_recipe_from_ingredients_instructions() {
        let data = serde_json::json!({
            "ingredients": ["flour", "sugar"], "instructions": "mix and bake"
        });
        assert_eq!(infer_entity_type("recipe:1", &data), EntityType::Recipe);
    }

    #[test]
    fn falls_back_to_key_prefix() {
        let data = serde_json::json!({"foo": "bar"});
        assert_eq!(infer_entity_type("user:1", &data), EntityType::Person);
        assert_eq!(infer_entity_type("post:1", &data), EntityType::Article);
        assert_eq!(infer_entity_type("unknown:1", &data), EntityType::WebPage);
    }

    #[test]
    fn key_to_entity_id_converts_correctly() {
        assert_eq!(key_to_entity_id("user:42"), "users/42");
        assert_eq!(key_to_entity_id("post:hello"), "posts/hello");
        assert_eq!(key_to_entity_id("product:abc-123"), "products/abc-123");
    }

    #[test]
    fn knowledge_graph_builds_from_db() {
        let db = nawa_db::DbEngine::open_in_memory();
        let _ = db.put("user:1", nawa_db::Value::from_json_str(
            r#"{"username":"admin","email":"a@b.com","password_hash":"x"}"#
        ).unwrap());
        let _ = db.put("post:1", nawa_db::Value::from_json_str(
            r#"{"headline":"Hello","author":"1","date_published":"2026-01-01"}"#
        ).unwrap());

        let graph = build_knowledge_graph(&db);
        assert!(graph.entity_count() >= 2);
        // Should have detected authoredBy relationship.
        assert!(graph.relationship_count() >= 1);
    }

    #[test]
    fn generate_jsonld_excludes_sensitive_fields() {
        let entity = Entity {
            id: "users/1".into(),
            entity_type: EntityType::Person,
            url: "/users/1".into(),
            properties: serde_json::json!({
                "username": "admin",
                "password_hash": "secret",
                "email": "a@b.com",
                "phone": "+1234567890",
                "name": "Admin User"
            }),
            last_modified: "2026-01-01".into(),
            importance: 0.7,
        };
        let jsonld = generate_jsonld(&entity);
        let s = jsonld.to_string();
        // Public fields should be present.
        assert!(s.contains("admin"));
        assert!(s.contains("Admin User"));
        // Private/sensitive fields must NEVER appear in SEO output.
        assert!(!s.contains("secret"), "password_hash value leaked!");
        assert!(!s.contains("password_hash"), "password_hash key leaked!");
        assert!(!s.contains("a@b.com"), "email leaked to SEO!");
        assert!(!s.contains("+1234567890"), "phone leaked to SEO!");
    }

    #[test]
    fn generate_jsonld_graph_has_at_graph() {
        let graph = KnowledgeGraph::new();
        let jsonld = generate_jsonld_graph(&graph);
        assert!(jsonld.get("@graph").is_some());
        assert!(jsonld.get("@context").is_some());
    }

    #[test]
    fn schema_urls_are_valid() {
        assert_eq!(EntityType::Person.schema_url(), "https://schema.org/Person");
        assert_eq!(EntityType::Article.schema_url(), "https://schema.org/Article");
    }

    #[test]
    fn importance_is_in_valid_range() {
        let data = serde_json::json!({});
        for et in [EntityType::Person, EntityType::Article, EntityType::Product,
                   EntityType::Event, EntityType::WebPage] {
            let imp = compute_importance(et, &data);
            assert!((0.0..=1.0).contains(&imp));
        }
    }

    #[test]
    fn skips_internal_keys_when_building_graph() {
        let db = nawa_db::DbEngine::open_in_memory();
        let _ = db.put("auth:session:1", nawa_db::Value::from_str("x"));
        let _ = db.put("seo:snapshot:/", nawa_db::Value::from_str("html"));
        let _ = db.put("user:1", nawa_db::Value::from_json_str(
            r#"{"username":"admin","email":"a@b.com","password_hash":"x"}"#
        ).unwrap());
        let graph = build_knowledge_graph(&db);
        // Only the user entity should be in the graph (auth: and seo: skipped).
        assert_eq!(graph.entity_count(), 1);
    }
}
