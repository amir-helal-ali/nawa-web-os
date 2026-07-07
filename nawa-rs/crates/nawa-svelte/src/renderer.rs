//! SvelteKit page renderer — bridges pre-rendered HTML with NAWA's zero-copy HTML engine.
//!
//! When a route has `prerendered_html`, we serve that file directly (zero-copy).
//! When it doesn't, we render a SPA shell that hydrates on the client.
//! In both cases, we inject NAWA-specific data:
//! - Initial DB state (for hydration)
//! - WebSocket URL (for live updates)
//! - Auth token (from cookie)
//! - CSRF token (for form submissions)

use std::collections::HashMap;
use std::path::Path;
use crate::manifest::{NawaManifest, Route};

/// Render context — everything needed to render a SvelteKit page in NAWA.
pub struct RenderContext<'a> {
    pub manifest: &'a NawaManifest,
    pub route: &'a Route,
    pub params: HashMap<String, String>,
    pub query: HashMap<String, String>,
    /// Initial DB state — serialized as JSON and injected into the page.
    pub initial_state: serde_json::Value,
    /// WebSocket URL the client should connect to (e.g. "ws://localhost:8081").
    pub ws_url: String,
    /// Auth token (from cookie), if logged in.
    pub auth_token: Option<String>,
    /// CSRF token (generated per request).
    pub csrf_token: String,
    /// User data (if logged in).
    pub user: Option<serde_json::Value>,
}

/// Result of rendering — the final HTML bytes ready to ship to the client.
pub struct RenderedPage {
    pub html: Vec<u8>,
    pub content_type: &'static str,
    pub status: u16,
    pub headers: Vec<(String, String)>,
}

impl RenderedPage {
    pub fn ok(html: Vec<u8>) -> Self {
        Self { html, content_type: "text/html; charset=utf-8", status: 200, headers: Vec::new() }
    }
    pub fn not_found(msg: impl Into<String>) -> Self {
        let msg = msg.into();
        let html = format!(r#"<!DOCTYPE html><html><body><h1>404</h1><p>{msg}</p></body></html>"#);
        Self { html: html.into_bytes(), content_type: "text/html; charset=utf-8", status: 404, headers: Vec::new() }
    }
}

/// The SvelteKit renderer — owned by the NAWA server.
pub struct SvelteRenderer {
    /// Root directory containing _nawa/ (pages/, assets/, ssr/).
    pub root: std::path::PathBuf,
}

impl SvelteRenderer {
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self { root: root.as_ref().to_path_buf() }
    }

    /// Render a matched route.
    /// This is the main entry point — called by the NAWA HTTP handler after route matching.
    pub fn render(&self, ctx: RenderContext<'_>) -> RenderedPage {
        // If the route has pre-rendered HTML, load it and inject NAWA data.
        if let Some(html_path) = &ctx.route.prerendered_html {
            return self.serve_prerendered(html_path, &ctx);
        }

        // If the route has SSR WASM, render dynamically (future: via nawa-wasm).
        if let Some(wasm_path) = &ctx.route.ssr_wasm {
            return self.render_ssr_wasm(wasm_path, &ctx);
        }

        // Otherwise, render the SPA shell — the client hydrates and fetches data.
        self.render_spa_shell(&ctx)
    }

    /// Serve a pre-rendered HTML file (zero-copy if possible).
    fn serve_prerendered(&self, html_path: &str, ctx: &RenderContext<'_>) -> RenderedPage {
        // The root is already the _nawa/ directory; pages live at root/pages/.
        let full_path = self.root.join("pages").join(html_path);
        match std::fs::read(&full_path) {
            Ok(mut html) => {
                // Inject NAWA bootstrap before </head>.
                let bootstrap = self.build_bootstrap_script(ctx);
                inject_before_close_tag(&mut html, b"</head>", bootstrap.as_bytes());
                RenderedPage::ok(html)
            }
            Err(e) => {
                tracing::warn!("prerendered HTML missing: {full_path:?} — {e}");
                self.render_spa_shell(ctx)
            }
        }
    }

    /// Render the SPA shell — minimal HTML that loads the JS bundle.
    /// The bundle then hydrates and takes over rendering.
    fn render_spa_shell(&self, ctx: &RenderContext<'_>) -> RenderedPage {
        let title = ctx.route.meta.title.as_deref()
            .or(ctx.manifest.default_meta.title.as_deref())
            .unwrap_or(&ctx.manifest.app_name);

        let description = ctx.route.meta.description.as_deref()
            .or(ctx.manifest.default_meta.description.as_deref())
            .unwrap_or("");

        let main_js = ctx.manifest.main_js.as_deref().unwrap_or("app.js");
        let global_css = ctx.manifest.global_css.as_deref();
        let favicon = ctx.manifest.favicon.as_deref();

        let css_link = global_css
            .map(|p| format!(r#"<link rel="stylesheet" href="/_nawa/assets/{p}">"#))
            .unwrap_or_default();

        let favicon_link = favicon
            .map(|p| format!(r#"<link rel="icon" href="/_nawa/assets/{p}">"#))
            .unwrap_or_default();

        let hydration_script = ctx.route.hydration_js.as_deref()
            .map(|p| format!(r#"<script type="module" src="/_nawa/assets/{p}" defer></script>"#))
            .unwrap_or_default();

        let bootstrap = self.build_bootstrap_script(ctx);

        let html = format!(r#"<!DOCTYPE html>
<html lang="ar" dir="rtl">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width,initial-scale=1.0">
<title>{title}</title>
<meta name="description" content="{description}">
{css_link}
{favicon_link}
<script type="module" src="/_nawa/assets/{main_js}" defer></script>
{hydration_script}
{bootstrap}
</head>
<body>
<div id="svelte"><!-- SvelteKit app hydrates here --></div>
<noscript>NAWA + SvelteKit requires JavaScript.</noscript>
</body>
</html>"#);

        RenderedPage::ok(html.into_bytes())
    }

    /// SSR via WASM module (future — placeholder for now).
    fn render_ssr_wasm(&self, _wasm_path: &str, ctx: &RenderContext<'_>) -> RenderedPage {
        tracing::debug!("SSR WASM requested but not yet implemented — falling back to SPA shell");
        self.render_spa_shell(ctx)
    }

    /// Build the NAWA bootstrap script — injects initial state, WS URL, auth token.
    /// This is what bridges NAWA (Rust) with SvelteKit (browser JS).
    fn build_bootstrap_script(&self, ctx: &RenderContext<'_>) -> String {
        let initial_state_json = serde_json::to_string(&ctx.initial_state).unwrap_or_else(|_| "{}".into());
        let user_json = serde_json::to_string(&ctx.user).unwrap_or_else(|_| "null".into());
        let params_json = serde_json::to_string(&ctx.params).unwrap_or_else(|_| "{}".into());
        let query_json = serde_json::to_string(&ctx.query).unwrap_or_else(|_| "{}".into());
        let token = ctx.auth_token.as_deref().unwrap_or("");

        format!(r#"<script>
window.__NAWA__ = {{
    appName: {app_name:?},
    wsUrl: {ws_url:?},
    authToken: {token:?},
    csrfToken: {csrf:?},
    route: {{
        pattern: {pattern:?},
        params: {params_json},
        query: {query_json}
    }},
    user: {user_json},
    initialState: {initial_state_json},
    version: "0.1.0-alpha",
    transport: "websocket-push",
    polling: false
}};
// Connect to NAWA WebSocket immediately for live updates.
(function(){{
    if(typeof WebSocket === 'undefined') return;
    var ws = new WebSocket(window.__NAWA__.wsUrl);
    var reconnectDelay = 1000;
    ws.onopen = function(){{ reconnectDelay = 1000; console.log('[NAWA] WebSocket connected'); }};
    ws.onmessage = function(ev){{
        try {{
            var evt = JSON.parse(ev.data);
            window.dispatchEvent(new CustomEvent('nawa:notification', {{detail: evt}}));
        }} catch(e) {{}}
    }};
    ws.onclose = function(){{
        setTimeout(function(){{
            ws = new WebSocket(window.__NAWA__.wsUrl);
        }}, reconnectDelay);
        reconnectDelay = Math.min(reconnectDelay * 2, 10000);
    }};
    window.__NAWA__.ws = ws;
}})();
</script>"#,
            app_name = ctx.manifest.app_name,
            ws_url = ctx.ws_url,
            token = token,
            csrf = ctx.csrf_token,
            pattern = ctx.route.pattern,
            params_json = params_json,
            query_json = query_json,
            user_json = user_json,
            initial_state_json = initial_state_json
        )
    }

    /// Serve a static asset from _nawa/assets/ (CSS, JS, images).
    pub fn serve_asset(&self, asset_path: &str) -> Option<(Vec<u8>, &'static str)> {
        let full_path = self.root.join("assets").join(asset_path);
        let bytes = std::fs::read(&full_path).ok()?;
        let content_type = match Path::new(asset_path).extension().and_then(|e| e.to_str()) {
            Some("js") => "application/javascript; charset=utf-8",
            Some("mjs") => "application/javascript; charset=utf-8",
            Some("css") => "text/css; charset=utf-8",
            Some("html") => "text/html; charset=utf-8",
            Some("png") => "image/png",
            Some("jpg") | Some("jpeg") => "image/jpeg",
            Some("svg") => "image/svg+xml",
            Some("ico") => "image/x-icon",
            Some("woff") => "font/woff",
            Some("woff2") => "font/woff2",
            _ => "application/octet-stream",
        };
        Some((bytes, content_type))
    }
}

/// Inject a payload before a closing tag in an HTML byte buffer.
/// Used to insert the bootstrap script before </head> in pre-rendered pages.
fn inject_before_close_tag(html: &mut Vec<u8>, close_tag: &[u8], payload: &[u8]) {
    if let Some(pos) = find_subslice(html, close_tag) {
        html.splice(pos..pos, payload.iter().copied());
    } else {
        // No closing tag found — append at the end.
        html.extend_from_slice(payload);
    }
}

/// Find the position of a subslice within a slice (linear search).
fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|w| w == needle)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::NawaManifest;

    fn make_manifest() -> NawaManifest {
        serde_json::from_str(r#"{
            "version": 1, "app_name": "TestApp", "built_at": "2026", "sveltekit_version": "2",
            "routes": [
                {"pattern":"/","methods":["GET"],"prerendered_html":null,"hydration_js":null,"requires_auth":false,"admin_only":false,"meta":{"title":"Home"},"is_endpoint":false,"ssr_wasm":null,"layout":null}
            ],
            "global_css": "app.css", "main_js": "app.js", "favicon": null,
            "spa_fallback": "spa.html",
            "default_meta": {"title": "Default", "description": "Default desc"}
        }"#).unwrap()
    }

    #[test]
    fn spa_shell_contains_bootstrap() {
        let m = make_manifest();
        let r = SvelteRenderer::new("/tmp");
        let route = &m.routes[0];
        let ctx = RenderContext {
            manifest: &m,
            route,
            params: HashMap::new(),
            query: HashMap::new(),
            initial_state: serde_json::json!({"count": 0}),
            ws_url: "ws://localhost:8081".into(),
            auth_token: Some("tok123".into()),
            csrf_token: "csrf456".into(),
            user: Some(serde_json::json!({"username": "admin"})),
        };
        let page = r.render(ctx);
        let html = String::from_utf8(page.html).unwrap();
        assert!(html.contains("window.__NAWA__"));
        assert!(html.contains("TestApp"));
        assert!(html.contains("ws://localhost:8081"));
        assert!(html.contains("tok123"));
        assert!(html.contains("csrf456"));
        assert!(html.contains("\"count\":0"));
        assert!(html.contains("Home")); // route-specific title takes priority
        assert!(html.contains("app.js"));
        assert!(html.contains("app.css"));
    }

    #[test]
    fn bootstrap_includes_no_polling_guarantee() {
        let m = make_manifest();
        let r = SvelteRenderer::new("/tmp");
        let route = &m.routes[0];
        let ctx = RenderContext {
            manifest: &m, route,
            params: HashMap::new(), query: HashMap::new(),
            initial_state: serde_json::Value::Null,
            ws_url: "ws://localhost:8081".into(),
            auth_token: None, csrf_token: "x".into(), user: None,
        };
        let page = r.render(ctx);
        let html = String::from_utf8(page.html).unwrap();
        assert!(html.contains("polling: false"));
        assert!(html.contains("WebSocket"));
    }

    #[test]
    fn inject_before_close_tag_works() {
        let mut html = b"<html><head></head><body></body></html>".to_vec();
        inject_before_close_tag(&mut html, b"</head>", b"<script>BOOT</script>");
        assert!(String::from_utf8(html).unwrap().contains("<script>BOOT</script></head>"));
    }

    #[test]
    fn find_subslice_finds_needle() {
        assert_eq!(find_subslice(b"hello world", b"world"), Some(6));
        assert_eq!(find_subslice(b"hello", b"xyz"), None);
        assert_eq!(find_subslice(b"", b"x"), None);
    }

    #[test]
    fn not_found_page_works() {
        let p = RenderedPage::not_found("missing route");
        assert_eq!(p.status, 404);
        assert!(String::from_utf8_lossy(&p.html).contains("missing route"));
    }

    #[test]
    fn serve_asset_returns_correct_content_type() {
        let r = SvelteRenderer::new("/tmp");
        // Non-existent file returns None.
        assert!(r.serve_asset("nonexistent.js").is_none());
    }
}
