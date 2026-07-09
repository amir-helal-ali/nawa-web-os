//! SvelteKit manifest — the bridge between SvelteKit build output and NAWA runtime.
//!
//! When a SvelteKit app is built with `adapter-nawa`, it produces a `manifest.json`
//! describing all routes, their pre-rendered HTML, their hydration scripts, and
//! metadata (load functions, actions, etc.). This module parses that manifest
//! and provides zero-copy route matching.

use std::collections::HashMap;
use std::path::Path;
use serde::{Deserialize, Serialize};

/// Root manifest — the entry point produced by adapter-nawa.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NawaManifest {
    /// Manifest format version (currently 1).
    pub version: u32,
    /// App name (from svelte.config.js).
    pub app_name: String,
    /// Build timestamp (ISO 8601).
    pub built_at: String,
    /// SvelteKit version used at build time.
    pub sveltekit_version: String,
    /// All registered routes, in declaration order.
    pub routes: Vec<Route>,
    /// Global CSS file path (relative to _nawa/assets/).
    pub global_css: Option<String>,
    /// Main JS bundle path (relative to _nawa/assets/).
    pub main_js: Option<String>,
    /// Favicon path (relative to _nawa/assets/).
    pub favicon: Option<String>,
    /// SPA fallback HTML path (relative to _nawa/pages/).
    /// Used when no route matches and SSR is disabled.
    pub spa_fallback: Option<String>,
    /// Default metadata (title, description).
    pub default_meta: MetaTags,
}

/// Metadata for a single SvelteKit route.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Route {
    /// Route pattern, e.g. "/", "/users/[id]", "/blog/[slug]".
    pub pattern: String,
    /// HTTP methods this route handles (GET, POST, etc.).
    pub methods: Vec<String>,
    /// Pre-rendered HTML file path (relative to _nawa/pages/), if any.
    /// If present, the route is served as a static HTML file (zero-copy).
    pub prerendered_html: Option<String>,
    /// Hydration JS chunk path (relative to _nawa/assets/), if any.
    /// If present, the browser hydrates the page into an interactive Svelte component.
    pub hydration_js: Option<String>,
    /// Whether this route requires authentication.
    pub requires_auth: bool,
    /// Whether this route is admin-only.
    pub admin_only: bool,
    /// Route metadata (title, description, OG tags).
    pub meta: MetaTags,
    /// Whether the route is a server-side endpoint (API route).
    /// If true, NAWA dispatches to the embedded handler instead of serving HTML.
    pub is_endpoint: bool,
    /// Optional: WASM module path for SSR rendering (relative to _nawa/ssr/).
    /// If present, NAWA renders the page dynamically via the WASM module.
    pub ssr_wasm: Option<String>,
    /// Layout slots (for nested layouts).
    pub layout: Option<String>,
}

/// HTML meta tags — used for SEO and social sharing.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MetaTags {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub og_image: Option<String>,
    #[serde(default)]
    pub canonical: Option<String>,
    #[serde(default)]
    pub extra: HashMap<String, String>,
}

/// A matched route with extracted parameters.
#[derive(Debug, Clone)]
pub struct MatchedRoute<'a> {
    pub route: &'a Route,
    pub params: HashMap<String, String>,
}

impl NawaManifest {
    /// Load a manifest from a JSON file on disk.
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let bytes = std::fs::read(path)
            .map_err(|e| anyhow::anyhow!("failed to read manifest {}: {e}", path.display()))?;
        let manifest: NawaManifest = serde_json::from_slice(&bytes)
            .map_err(|e| anyhow::anyhow!("failed to parse manifest JSON: {e}"))?;
        tracing::debug!(
            "loaded manifest v{} for '{}' — {} routes",
            manifest.version, manifest.app_name, manifest.routes.len()
        );
        Ok(manifest)
    }

    /// Parse a manifest from raw JSON bytes (useful for embedding in binary).
    pub fn from_bytes(bytes: &[u8]) -> anyhow::Result<Self> {
        serde_json::from_slice(bytes)
            .map_err(|e| anyhow::anyhow!("invalid manifest JSON: {e}"))
    }

    /// Match a request path against all routes.
    /// Returns the first matching route with extracted parameters.
    ///
    /// Supports SvelteKit-style dynamic segments:
    /// - `[id]` → named parameter
    /// - `[...rest]` → catch-all (greedy)
    /// - `[[optional]]` → optional segment (matches presence or absence)
    ///
    /// # Examples
    /// ```
    /// use nawa_svelte::manifest::NawaManifest;
    /// let m: NawaManifest = serde_json::from_str(r#"{
    ///     "version":1,"app_name":"x","built_at":"t","sveltekit_version":"1",
    ///     "routes":[],"default_meta":{}
    /// }"#).unwrap();
    /// let r = m.match_route("/users/42");
    /// ```
    pub fn match_route(&self, path: &str) -> Option<MatchedRoute<'_>> {
        let path = path.split('?').next().unwrap_or(path);
        let path_segments: Vec<&str> = path
            .split('/')
            .filter(|s| !s.is_empty())
            .collect();

        for route in &self.routes {
            if let Some(params) = match_pattern(&route.pattern, &path_segments) {
                return Some(MatchedRoute { route, params });
            }
        }
        None
    }

    /// Iterate over all routes (for introspection / admin UI).
    pub fn iter_routes(&self) -> impl Iterator<Item = &Route> {
        self.routes.iter()
    }

    /// Total route count.
    pub fn route_count(&self) -> usize {
        self.routes.len()
    }
}

/// Match a SvelteKit-style pattern against path segments.
/// Returns Some(params) if matched, None otherwise.
fn match_pattern(pattern: &str, path_segments: &[&str]) -> Option<HashMap<String, String>> {
    let pattern_segments: Vec<&str> = pattern
        .split('/')
        .filter(|s| !s.is_empty())
        .collect();

    let mut params = HashMap::new();
    let mut p_iter = pattern_segments.iter().peekable();
    let mut s_iter = path_segments.iter().peekable();

    while let Some(pseg) = p_iter.next() {
        // Catch-all: [...rest]
        if let Some(name) = pseg.strip_prefix("[...").and_then(|s| s.strip_suffix(']')) {
            // Consume all remaining path segments.
            let rest: Vec<&str> = s_iter.by_ref().copied().collect();
            params.insert(name.to_string(), rest.join("/"));
            return Some(params);
        }

        // Optional: [[optional]]
        if let Some(name) = pseg.strip_prefix("[[").and_then(|s| s.strip_suffix("]]")) {
            // If the next path segment exists and the next pattern segment matches it,
            // we treat the optional as present.
            match (s_iter.peek(), p_iter.peek()) {
                (Some(_), _) => {
                    if let Some(seg) = s_iter.next() {
                        params.insert(name.to_string(), seg.to_string());
                    }
                }
                (None, _) => {
                    // Optional segment absent — that's fine.
                }
            }
            continue;
        }

        // Named: [id]
        if pseg.starts_with('[') && pseg.ends_with(']') {
            let name = &pseg[1..pseg.len() - 1];
            match s_iter.next() {
                Some(seg) => {
                    params.insert(name.to_string(), seg.to_string());
                }
                None => return None, // pattern expects a segment, but path is shorter
            }
            continue;
        }

        // Literal segment
        match s_iter.next() {
            Some(seg) if *seg == *pseg => continue,
            _ => return None,
        }
    }

    // If there are leftover path segments, the pattern didn't fully match —
    // UNLESS the last pattern segment was a catch-all (handled above).
    if s_iter.next().is_some() {
        return None;
    }

    Some(params)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_manifest() -> NawaManifest {
        serde_json::from_str(r#"{
            "version": 1,
            "app_name": "test",
            "built_at": "2026-01-01",
            "sveltekit_version": "2.0.0",
            "routes": [
                {"pattern":"/","methods":["GET"],"prerendered_html":"index.html","hydration_js":"index.js","requires_auth":false,"admin_only":false,"meta":{},"is_endpoint":false,"ssr_wasm":null,"layout":null},
                {"pattern":"/users/[id]","methods":["GET"],"prerendered_html":null,"hydration_js":"users/[id].js","requires_auth":true,"admin_only":false,"meta":{},"is_endpoint":false,"ssr_wasm":null,"layout":null},
                {"pattern":"/blog/[...slug]","methods":["GET"],"prerendered_html":null,"hydration_js":null,"requires_auth":false,"admin_only":false,"meta":{},"is_endpoint":false,"ssr_wasm":null,"layout":null},
                {"pattern":"/api/data","methods":["GET","POST"],"prerendered_html":null,"hydration_js":null,"requires_auth":false,"admin_only":false,"meta":{},"is_endpoint":true,"ssr_wasm":null,"layout":null},
                {"pattern":"/admin/[[section]]","methods":["GET"],"prerendered_html":null,"hydration_js":null,"requires_auth":true,"admin_only":true,"meta":{},"is_endpoint":false,"ssr_wasm":null,"layout":null}
            ],
            "global_css": "app.css",
            "main_js": "app.js",
            "favicon": "favicon.ico",
            "spa_fallback": "spa.html",
            "default_meta": {}
        }"#).unwrap()
    }

    #[test]
    fn matches_root_route() {
        let m = sample_manifest();
        let r = m.match_route("/").unwrap();
        assert_eq!(r.route.pattern, "/");
        assert!(r.params.is_empty());
    }

    #[test]
    fn matches_named_param() {
        let m = sample_manifest();
        let r = m.match_route("/users/42").unwrap();
        assert_eq!(r.route.pattern, "/users/[id]");
        assert_eq!(r.params.get("id"), Some(&"42".to_string()));
    }

    #[test]
    fn matches_catch_all() {
        let m = sample_manifest();
        let r = m.match_route("/blog/2024/01/hello-world").unwrap();
        assert_eq!(r.route.pattern, "/blog/[...slug]");
        assert_eq!(r.params.get("slug"), Some(&"2024/01/hello-world".to_string()));
    }

    #[test]
    fn matches_optional_segment_present() {
        let m = sample_manifest();
        let r = m.match_route("/admin/settings").unwrap();
        assert_eq!(r.route.pattern, "/admin/[[section]]");
        assert_eq!(r.params.get("section"), Some(&"settings".to_string()));
    }

    #[test]
    fn matches_optional_segment_absent() {
        let m = sample_manifest();
        let r = m.match_route("/admin").unwrap();
        assert_eq!(r.route.pattern, "/admin/[[section]]");
        assert!(!r.params.contains_key("section"));
    }

    #[test]
    fn returns_none_for_unknown_route() {
        let m = sample_manifest();
        assert!(m.match_route("/nonexistent").is_none());
    }

    #[test]
    fn ignores_query_string_in_matching() {
        let m = sample_manifest();
        let r = m.match_route("/users/42?tab=posts").unwrap();
        assert_eq!(r.params.get("id"), Some(&"42".to_string()));
    }

    #[test]
    fn route_count_is_correct() {
        let m = sample_manifest();
        assert_eq!(m.route_count(), 5);
    }

    #[test]
    fn endpoint_routes_are_marked() {
        let m = sample_manifest();
        let r = m.match_route("/api/data").unwrap();
        assert!(r.route.is_endpoint);
    }

    #[test]
    fn admin_routes_are_marked() {
        let m = sample_manifest();
        let r = m.match_route("/admin").unwrap();
        assert!(r.route.admin_only);
        assert!(r.route.requires_auth);
    }

    #[test]
    fn manifest_parses_from_bytes() {
        let bytes = br#"{"version":1,"app_name":"x","built_at":"t","sveltekit_version":"1","routes":[],"default_meta":{}}"#;
        let m = NawaManifest::from_bytes(bytes).unwrap();
        assert_eq!(m.app_name, "x");
    }
}
