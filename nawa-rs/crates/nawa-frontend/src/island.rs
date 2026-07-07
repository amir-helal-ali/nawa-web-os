//! Island hydration system — interactive components that ship minimal JS.
//!
//! ## Architecture
//!
//! 1. Server renders full HTML (SSR) — all islands included.
//! 2. Each island has a `data-island` attribute with a unique ID.
//! 3. Client runtime (3KB WASM) reads `data-island` attributes.
//! 4. Only interactive islands get hydrated — rest stays static.
//!
//! This means: a page with 50KB HTML + 1 interactive form = 50KB + 3KB WASM,
//! instead of a 200KB SPA bundle.

use crate::html::{HtmlElement, HtmlNode};
use std::collections::HashMap;

/// Properties passed to an island (JSON-serializable).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IslandProps {
    data: serde_json::Value,
}

impl IslandProps {
    /// Create from any serializable value.
    pub fn new(data: impl serde::Serialize) -> Self {
        Self {
            data: serde_json::to_value(data).unwrap_or(serde_json::Value::Null),
        }
    }

    /// Create from JSON string.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        Ok(Self {
            data: serde_json::from_str(json)?,
        })
    }

    /// Serialize to JSON string.
    pub fn to_json(&self) -> String {
        serde_json::to_string(&self.data).unwrap_or_else(|_| "null".into())
    }

    /// Get the raw value.
    pub fn value(&self) -> &serde_json::Value {
        &self.data
    }
}

/// An island — a self-contained interactive component.
pub struct Island {
    /// Unique island ID (used for hydration).
    pub id: String,
    /// Component name (matches client-side registry).
    pub component: String,
    /// Properties passed to the component.
    pub props: IslandProps,
    /// The SSR-rendered HTML content.
    pub ssr_content: Vec<HtmlNode>,
}

impl Island {
    /// Create a new island.
    pub fn new(id: impl Into<String>, component: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            component: component.into(),
            props: IslandProps::new(serde_json::Value::Null),
            ssr_content: Vec::new(),
        }
    }

    /// Set props.
    pub fn props(mut self, props: impl serde::Serialize) -> Self {
        self.props = IslandProps::new(props);
        self
    }

    /// Set props from JSON.
    pub fn props_json(mut self, json: &str) -> Self {
        if let Ok(p) = IslandProps::from_json(json) {
            self.props = p;
        }
        self
    }

    /// Add SSR content (what the server renders).
    pub fn content(mut self, node: impl Into<HtmlNode>) -> Self {
        self.ssr_content.push(node.into());
        self
    }

    /// Add multiple content nodes.
    pub fn content_nodes(mut self, nodes: Vec<HtmlNode>) -> Self {
        self.ssr_content.extend(nodes);
        self
    }

    /// Render to HTML element (with data-island attributes).
    pub fn to_element(&self) -> HtmlElement {
        let mut el = HtmlElement::new("div")
            .attr("data-island", &self.id)
            .attr("data-component", &self.component)
            .attr("data-props", self.props.to_json());

        for node in &self.ssr_content {
            el = el.child(node.clone());
        }

        el
    }

    /// Render to HTML string.
    pub fn render(&self) -> String {
        self.to_element().render()
    }
}

/// Registry of all islands on a page.
pub struct IslandRegistry {
    islands: HashMap<String, Island>,
}

impl IslandRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            islands: HashMap::new(),
        }
    }

    /// Register an island.
    pub fn register(&mut self, island: Island) {
        self.islands.insert(island.id.clone(), island);
    }

    /// Get an island by ID.
    pub fn get(&self, id: &str) -> Option<&Island> {
        self.islands.get(id)
    }

    /// Number of registered islands.
    pub fn len(&self) -> usize {
        self.islands.len()
    }

    /// Is the registry empty?
    pub fn is_empty(&self) -> bool {
        self.islands.is_empty()
    }

    /// Render all islands as HTML elements.
    pub fn render_all(&self) -> Vec<HtmlNode> {
        self.islands.values().map(|i| HtmlNode::Element(i.to_element())).collect()
    }

    /// Generate the client-side hydration script.
    ///
    /// This script reads all `data-island` elements and hydrates them.
    /// In production, this would be a 3KB WASM module.
    pub fn hydration_script(&self) -> String {
        let islands_json: Vec<serde_json::Value> = self.islands.values().map(|i| {
            serde_json::json!({
                "id": i.id,
                "component": i.component,
                "props": i.props.value(),
            })
        }).collect();

        let json_str = serde_json::to_string(&islands_json).unwrap_or_else(|_| "[]".into());

        format!(
            r#"<script>
window.__NAWA_ISLANDS__ = {json};
(function() {{
    var islands = document.querySelectorAll('[data-island]');
    islands.forEach(function(el) {{
        var component = el.getAttribute('data-component');
        var props = el.getAttribute('data-props');
        console.log('[NAWA] Hydrating island:', component, 'with props:', props);
        el.setAttribute('data-hydrated', 'true');
    }});
}})();
</script>"#,
            json = json_str
        )
    }
}

impl Default for IslandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::html::{h1, p};

    #[test]
    fn island_creation() {
        let island = Island::new("counter-1", "Counter")
            .props(serde_json::json!({"initial": 0}))
            .content(h1().text("Count: 0"));

        let html = island.render();
        assert!(html.contains("data-island=\"counter-1\""));
        assert!(html.contains("data-component=\"Counter\""));
        assert!(html.contains("data-props="));
        assert!(html.contains("<h1>Count: 0</h1>"));
    }

    #[test]
    fn island_props_json() {
        let island = Island::new("form-1", "Form")
            .props_json(r#"{"action":"/submit"}"#);

        let html = island.render();
        assert!(html.contains("action"));
        assert!(html.contains("submit"));
    }

    #[test]
    fn registry_multiple_islands() {
        let mut registry = IslandRegistry::new();
        registry.register(
            Island::new("nav", "NavBar").content(p().text("Navigation"))
        );
        registry.register(
            Island::new("form", "ContactForm").props(serde_json::json!({"fields": ["name", "email"]}))
        );

        assert_eq!(registry.len(), 2);
        let nodes = registry.render_all();
        assert_eq!(nodes.len(), 2);
    }

    #[test]
    fn hydration_script_generated() {
        let mut registry = IslandRegistry::new();
        registry.register(Island::new("counter", "Counter"));

        let script = registry.hydration_script();
        assert!(script.contains("<script>"));
        assert!(script.contains("__NAWA_ISLANDS__"));
        assert!(script.contains("counter"));
        assert!(script.contains("querySelectorAll"));
    }

    #[test]
    fn empty_registry() {
        let registry = IslandRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.render_all().len(), 0);
    }
}
