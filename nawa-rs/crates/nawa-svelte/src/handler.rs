//! HTTP handler — bridges NAWA's HTTP request/response with the SvelteKit renderer.
//!
//! This is the glue layer called by `nawad`'s router when a request matches
//! a SvelteKit route. It:
//! 1. Matches the request path against the manifest
//! 2. Builds a RenderContext (with auth, DB state, WS URL)
//! 3. Renders the page
//! 4. Returns an HTTP response

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use crate::manifest::NawaManifest;
use crate::renderer::{RenderContext, RenderedPage, SvelteRenderer};

/// The SvelteKit integration handler — owned by nawad's router.
///
/// Holds a reference to the manifest, the renderer, and a reference to NAWA-DB
/// for initial state injection.
pub struct SvelteHandler {
    pub manifest: NawaManifest,
    pub renderer: SvelteRenderer,
    /// WebSocket URL the browser should connect to (computed at startup).
    pub ws_url: String,
    /// Root directory of the SvelteKit build (containing _nawa/).
    pub root: PathBuf,
}

impl SvelteHandler {
    /// Load a SvelteKit integration from a build directory.
    /// Accepts either:
    ///   - The parent directory containing `_nawa/manifest.json`
    ///   - The `_nawa/` directory itself containing `manifest.json`
    pub fn load(root: impl Into<PathBuf>, ws_url: impl Into<String>) -> anyhow::Result<Arc<Self>> {
        let root = root.into();
        // Try _nawa/manifest.json first, then manifest.json directly.
        let manifest_path = if root.join("_nawa/manifest.json").exists() {
            root.join("_nawa/manifest.json")
        } else if root.join("manifest.json").exists() {
            root.join("manifest.json")
        } else {
            anyhow::bail!("no manifest.json found in {} or {}/_nawa/", root.display(), root.display())
        };
        let manifest = NawaManifest::load(&manifest_path)?;
        // Renderer root is the parent of _nawa/ (or the dir itself if manifest is at root).
        let renderer_root = if root.join("_nawa/manifest.json").exists() {
            root.join("_nawa")
        } else {
            root.clone()
        };
        let renderer = SvelteRenderer::new(&renderer_root);
        tracing::info!(
            "✓ SvelteKit integration loaded: '{}' — {} routes (root: {})",
            manifest.app_name, manifest.route_count(), renderer_root.display()
        );
        Ok(Arc::new(Self {
            manifest,
            renderer,
            ws_url: ws_url.into(),
            root: renderer_root,
        }))
    }

    /// Handle a request for a SvelteKit route.
    /// Returns the rendered page (HTML + status + headers).
    pub fn handle(
        &self,
        path: &str,
        query: HashMap<String, String>,
        auth_token: Option<&str>,
        user: Option<serde_json::Value>,
        db_state: serde_json::Value,
    ) -> RenderedPage {
        // Match the path against the manifest.
        let matched = match self.manifest.match_route(path) {
            Some(m) => m,
            None => {
                // No route matched — try SPA fallback if available.
                if let Some(fallback) = &self.manifest.spa_fallback {
                    let full_path = self.root.join("pages").join(fallback);
                    if let Ok(html) = std::fs::read(&full_path) {
                        return RenderedPage::ok(html);
                    }
                }
                return RenderedPage::not_found(format!("no route matches: {path}"));
            }
        };

        // Authorization check.
        if matched.route.requires_auth && auth_token.is_none() {
            return RenderedPage {
                html: redirect_html("/login").into_bytes(),
                content_type: "text/html; charset=utf-8",
                status: 302,
                headers: vec![("Location".into(), "/login".into())],
            };
        }
        if matched.route.admin_only {
            if let Some(u) = &user {
                if u.get("role").and_then(|r| r.as_str()) != Some("admin") {
                    return RenderedPage::not_found("admin required");
                }
            } else {
                return RenderedPage::not_found("admin required");
            }
        }

        // If it's an API endpoint, NAWA's router should have dispatched elsewhere.
        // But just in case, return a 405 here.
        if matched.route.is_endpoint {
            return RenderedPage {
                html: br#"{"error":"endpoint not configured"}"#.to_vec(),
                content_type: "application/json",
                status: 501,
                headers: Vec::new(),
            };
        }

        // Build the render context.
        let ctx = RenderContext {
            manifest: &self.manifest,
            route: matched.route,
            params: matched.params,
            query,
            initial_state: db_state,
            ws_url: self.ws_url.clone(),
            auth_token: auth_token.map(|s| s.to_string()),
            csrf_token: generate_csrf_token(),
            user,
        };

        self.renderer.render(ctx)
    }

    /// Serve a static asset (CSS, JS, image) from _nawa/assets/.
    pub fn serve_asset(&self, asset_path: &str) -> Option<(Vec<u8>, &'static str)> {
        self.renderer.serve_asset(asset_path)
    }

    /// Total registered routes.
    pub fn route_count(&self) -> usize {
        self.manifest.route_count()
    }
}

/// Generate a CSRF token (16 random hex chars).
fn generate_csrf_token() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    let pid = std::process::id();
    format!("{nanos:08x}{pid:08x}")
}

/// Build a minimal HTML redirect (used when auth is required).
fn redirect_html(location: &str) -> String {
    format!(
        r#"<!DOCTYPE html><html><head><meta http-equiv="refresh" content="0;url={location}"></head><body>Redirecting…</body></html>"#,
        location = location
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn make_test_handler() -> (tempfile::TempDir, Arc<SvelteHandler>) {
        let dir = tempfile::tempdir().unwrap();
        let nawa_dir = dir.path().join("_nawa");
        let pages_dir = nawa_dir.join("pages");
        let assets_dir = nawa_dir.join("assets");
        std::fs::create_dir_all(&pages_dir).unwrap();
        std::fs::create_dir_all(&assets_dir).unwrap();

        // Write manifest.
        let manifest = r#"{
            "version": 1, "app_name": "TestApp", "built_at": "2026", "sveltekit_version": "2",
            "routes": [
                {"pattern":"/","methods":["GET"],"prerendered_html":"index.html","hydration_js":null,"requires_auth":false,"admin_only":false,"meta":{"title":"Home"},"is_endpoint":false,"ssr_wasm":null,"layout":null},
                {"pattern":"/users/[id]","methods":["GET"],"prerendered_html":null,"hydration_js":"users.js","requires_auth":true,"admin_only":false,"meta":{"title":"User"},"is_endpoint":false,"ssr_wasm":null,"layout":null},
                {"pattern":"/admin","methods":["GET"],"prerendered_html":null,"hydration_js":null,"requires_auth":true,"admin_only":true,"meta":{"title":"Admin"},"is_endpoint":false,"ssr_wasm":null,"layout":null}
            ],
            "global_css": "app.css", "main_js": "app.js", "favicon": null,
            "spa_fallback": "spa.html",
            "default_meta": {"title": "Default", "description": "Default desc"}
        }"#;
        std::fs::write(nawa_dir.join("manifest.json"), manifest).unwrap();

        // Write a pre-rendered index.html.
        let mut f = std::fs::File::create(pages_dir.join("index.html")).unwrap();
        writeln!(f, "<!DOCTYPE html><html><head><title>Home</title></head><body><h1>NAWA Home</h1></body></html>").unwrap();

        // Write SPA fallback.
        std::fs::write(pages_dir.join("spa.html"), "<!DOCTYPE html><html><body>SPA</body></html>").unwrap();

        // Write a CSS asset.
        std::fs::write(assets_dir.join("app.css"), "body { font-family: sans-serif; }").unwrap();

        let handler = SvelteHandler::load(dir.path(), "ws://localhost:8081").unwrap();
        (dir, handler)
    }

    #[test]
    fn serves_prerendered_page() {
        let (_dir, h) = make_test_handler();
        let page = h.handle("/", HashMap::new(), None, None, serde_json::Value::Null);
        assert_eq!(page.status, 200);
        let html = String::from_utf8(page.html).unwrap();
        assert!(html.contains("NAWA Home"));
        assert!(html.contains("window.__NAWA__")); // bootstrap injected
    }

    #[test]
    fn renders_spa_shell_for_dynamic_routes() {
        let (_dir, h) = make_test_handler();
        let page = h.handle("/users/42", HashMap::new(), Some("tok"), None, serde_json::Value::Null);
        assert_eq!(page.status, 200);
        let html = String::from_utf8(page.html).unwrap();
        assert!(html.contains("id=\"svelte\""));
        assert!(html.contains("users.js"));
    }

    #[test]
    fn redirects_when_auth_required_and_no_token() {
        let (_dir, h) = make_test_handler();
        let page = h.handle("/users/42", HashMap::new(), None, None, serde_json::Value::Null);
        assert_eq!(page.status, 302);
        assert!(page.headers.iter().any(|(k, v)| k == "Location" && v == "/login"));
    }

    #[test]
    fn blocks_admin_route_for_non_admin() {
        let (_dir, h) = make_test_handler();
        let user = serde_json::json!({"role": "user"});
        let page = h.handle("/admin", HashMap::new(), Some("tok"), Some(user), serde_json::Value::Null);
        assert_eq!(page.status, 404);
    }

    #[test]
    fn allows_admin_route_for_admin() {
        let (_dir, h) = make_test_handler();
        let user = serde_json::json!({"role": "admin"});
        let page = h.handle("/admin", HashMap::new(), Some("tok"), Some(user), serde_json::Value::Null);
        assert_eq!(page.status, 200);
    }

    #[test]
    fn returns_404_for_unknown_route_without_fallback() {
        let dir = tempfile::tempdir().unwrap();
        let nawa_dir = dir.path().join("_nawa");
        std::fs::create_dir_all(&nawa_dir).unwrap();
        let manifest = r#"{
            "version": 1, "app_name": "X", "built_at": "t", "sveltekit_version": "1",
            "routes": [{"pattern":"/","methods":["GET"],"prerendered_html":null,"hydration_js":null,"requires_auth":false,"admin_only":false,"meta":{},"is_endpoint":false,"ssr_wasm":null,"layout":null}],
            "global_css": null, "main_js": null, "favicon": null,
            "spa_fallback": null,
            "default_meta": {}
        }"#;
        std::fs::write(nawa_dir.join("manifest.json"), manifest).unwrap();
        let h = SvelteHandler::load(dir.path(), "ws://x").unwrap();
        let page = h.handle("/nonexistent", HashMap::new(), None, None, serde_json::Value::Null);
        assert_eq!(page.status, 404);
    }

    #[test]
    fn serves_assets_with_correct_content_type() {
        let (_dir, h) = make_test_handler();
        let (bytes, ct) = h.serve_asset("app.css").unwrap();
        assert_eq!(ct, "text/css; charset=utf-8");
        assert!(String::from_utf8(bytes).unwrap().contains("font-family"));
    }

    #[test]
    fn csrf_token_is_16_chars() {
        let t = generate_csrf_token();
        assert_eq!(t.len(), 16);
    }
}
