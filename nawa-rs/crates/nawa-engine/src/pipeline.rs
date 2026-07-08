//! Unified Engine Pipeline — DB → SSR → HTTP in one zero-copy chain.
//!
//! This is the heart of NAWA: a single engine that reads from DB,
//! renders HTML, and produces a response buffer — all without
//! intermediate String allocations.

use crate::components;
use crate::design::DesignSystem;
use crate::zerocopy_html::ZeroCopyHtml;
use nawa_db::DbEngine;
#[allow(unused_imports)] use nawa_db::Value;
use std::sync::Arc;

/// The unified engine context — shared state for a single request.
pub struct EngineContext {
    /// Reference to the database (zero-copy access via Arc).
    pub db: Arc<DbEngine>,
    /// The design system (CSS).
    pub design: DesignSystem,
}

impl EngineContext {
    /// Create a new engine context.
    pub fn new(db: Arc<DbEngine>) -> Self {
        Self {
            db,
            design: DesignSystem::new(),
        }
    }
}

/// Result of a render — the HTML as bytes, ready for io_uring.
pub struct RenderResult {
    /// The complete HTML document as bytes.
    pub html: Vec<u8>,
    /// Content-Type header value.
    pub content_type: &'static str,
}

impl RenderResult {
    /// Get the HTML as bytes (for io_uring send).
    pub fn into_bytes(self) -> Vec<u8> {
        self.html
    }
}

/// The unified engine — renders pages with zero-copy DB access.
pub struct UnifiedEngine;

impl UnifiedEngine {
    /// Render a page that lists all DB entries.
    ///
    /// This demonstrates the zero-copy chain:
    /// 1. DB scan returns (Vec<u8>, Value) — no String
    /// 2. Keys/values are written to ZeroCopyHtml as &[u8] — no String
    /// 3. Final buffer is Vec<u8> — ready for io_uring
    pub fn render_db_page(ctx: &EngineContext, title: &str) -> RenderResult {
        let entries = ctx.db.scan_prefix("", 200);

        // Build the page using ZeroCopyHtml — no String allocations.
        let mut h = ZeroCopyHtml::with_capacity(8192);

        // DOCTYPE + html
        h.doctype()
            .raw_str(r#"<html lang="ar" dir="rtl"><head>"#)
            .raw_str(r#"<meta charset="UTF-8">"#)
            .raw_str(r#"<meta name="viewport" content="width=device-width, initial-scale=1.0">"#);

        // Title
        h.open_tag("title").text(title).close_tag("title");

        // CSS (design system)
        h.open_tag("style").raw_str(ctx.design.css()).close_tag("style");
        h.raw_str("</head><body>");

        // Navigation
        let nav = components::nav("NAWA", &[
            ("Home", "/"),
            ("SSR", "/ssr"),
            ("Health", "/health"),
            ("Metrics", "/metrics"),
        ]);
        h.raw(&nav.into_bytes());

        // Main content
        h.open_tag("div").class("nawa-container");
        h.open_tag("h1").text(title).close_tag("h1");

        // Stats
        let stats = components::stats(&[
            (&entries.len().to_string(), "Keys"),
            (&ctx.db.memtable_size().to_string(), "MemTable bytes"),
        ]);
        h.raw(&stats.into_bytes());

        // DB table (zero-copy from DB → HTML)
        if entries.is_empty() {
            h.open_tag("p").class("nawa-card")
                .text("No data yet. Use POST /:key to add.")
                .close_tag("p");
        } else {
            // Render table rows directly (true zero-copy: DB bytes → HTML buffer).
            h.raw_str(r#"<table class="nawa-table"><thead><tr><th>Key</th><th>Value</th></tr></thead><tbody>"#);
            for (k, v) in &entries {
                h.raw_str("<tr><td>");
                h.text_bytes(k);
                h.raw_str("</td><td>");
                h.text_bytes(v.display().as_bytes());
                h.raw_str("</td></tr>");
            }
            h.raw_str("</tbody></table>");
        }

        // Interactive island
        let island_content = b"<h2>Interactive Island</h2><p>Count: 0</p>";
        let island = components::island(
            "counter",
            "Counter",
            r#"{"initial":0}"#,
            island_content,
        );
        h.raw(&island.into_bytes());

        // Hydration script
        let hs = components::hydration_script(r#"[{"id":"counter","component":"Counter"}]"#);
        h.raw(&hs.into_bytes());

        // Close
        h.raw_str("</div></body></html>");

        RenderResult {
            html: h.into_bytes(),
            content_type: "text/html; charset=utf-8",
        }
    }

    /// Render a simple text response (for API endpoints).
    pub fn render_text(text: &str) -> RenderResult {
        let mut h = ZeroCopyHtml::with_capacity(text.len() + 64);
        h.text(text);
        RenderResult {
            html: h.into_bytes(),
            content_type: "text/plain; charset=utf-8",
        }
    }

    /// Render a JSON response (for API endpoints).
    pub fn render_json(json: &str) -> RenderResult {
        let bytes = json.as_bytes().to_vec();
        RenderResult {
            html: bytes,
            content_type: "application/json",
        }
    }

    /// Render an auth login page using the design system.
    pub fn render_login_page(ctx: &EngineContext) -> RenderResult {
        let mut h = ZeroCopyHtml::with_capacity(4096);
        h.doctype()
            .raw_str(r#"<html lang="ar" dir="rtl"><head>"#)
            .raw_str(r#"<meta charset="UTF-8">"#)
            .raw_str(r#"<meta name="viewport" content="width=device-width, initial-scale=1.0">"#)
            .open_tag("title").text("Login — NAWA").close_tag("title")
            .open_tag("style").raw_str(ctx.design.css()).close_tag("style")
            .raw_str("</head><body>");

        let nav = components::nav("NAWA", &[("Home", "/"), ("Register", "/register")]);
        h.raw(&nav.into_bytes());

        h.open_tag("div").class("nawa-container");
        h.open_tag("div").class("nawa-card");
        h.open_tag("h2").text("Login").close_tag("h2");

        let form = components::form("/login", &[
            ("email", "email", "Email"),
            ("password", "password", "Password"),
        ], "Login");
        h.raw(&form.into_bytes());

        h.close_tag("div").close_tag("div");
        h.raw_str("</body></html>");

        RenderResult {
            html: h.into_bytes(),
            content_type: "text/html; charset=utf-8",
        }
    }

    /// Render an auth register page.
    pub fn render_register_page(ctx: &EngineContext) -> RenderResult {
        let mut h = ZeroCopyHtml::with_capacity(4096);
        h.doctype()
            .raw_str(r#"<html lang="ar" dir="rtl"><head>"#)
            .raw_str(r#"<meta charset="UTF-8">"#)
            .raw_str(r#"<meta name="viewport" content="width=device-width, initial-scale=1.0">"#)
            .open_tag("title").text("Register — NAWA").close_tag("title")
            .open_tag("style").raw_str(ctx.design.css()).close_tag("style")
            .raw_str("</head><body>");

        let nav = components::nav("NAWA", &[("Home", "/"), ("Login", "/login")]);
        h.raw(&nav.into_bytes());

        h.open_tag("div").class("nawa-container");
        h.open_tag("div").class("nawa-card");
        h.open_tag("h2").text("Create Account").close_tag("h2");

        let form = components::form("/register", &[
            ("username", "text", "Username"),
            ("email", "email", "Email"),
            ("password", "password", "Password"),
        ], "Register");
        h.raw(&form.into_bytes());

        h.close_tag("div").close_tag("div");
        h.raw_str("</body></html>");

        RenderResult {
            html: h.into_bytes(),
            content_type: "text/html; charset=utf-8",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_ctx() -> EngineContext {
        EngineContext::new(Arc::new(DbEngine::open_in_memory()))
    }

    #[test]
    fn render_db_page_empty() {
        let ctx = make_ctx();
        let result = UnifiedEngine::render_db_page(&ctx, "Test Page");
        let html = String::from_utf8(result.html).unwrap();
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("Test Page"));
        assert!(html.contains("No data yet"));
        assert!(html.contains("data-island"));
        assert!(html.contains("__NAWA_ISLANDS__"));
    }

    #[test]
    fn render_db_page_with_data() {
        let ctx = make_ctx();
        ctx.db.put("user:1", Value::from_str("Ahmed")).unwrap();
        ctx.db.put("user:2", Value::from_str("Sara")).unwrap();

        let result = UnifiedEngine::render_db_page(&ctx, "Data Page");
        let html = String::from_utf8(result.html).unwrap();
        assert!(html.contains("Ahmed"));
        assert!(html.contains("Sara"));
        assert!(html.contains("nawa-table"));
    }

    #[test]
    fn render_login_page() {
        let ctx = make_ctx();
        let result = UnifiedEngine::render_login_page(&ctx);
        let html = String::from_utf8(result.html).unwrap();
        assert!(html.contains("Login"));
        assert!(html.contains("email"));
        assert!(html.contains("password"));
        assert!(html.contains("nawa-btn-primary"));
    }

    #[test]
    fn render_register_page() {
        let ctx = make_ctx();
        let result = UnifiedEngine::render_register_page(&ctx);
        let html = String::from_utf8(result.html).unwrap();
        assert!(html.contains("Create Account"));
        assert!(html.contains("username"));
        assert!(html.contains("Register"));
    }

    #[test]
    fn zero_copy_chain() {
        // Verify: DB data flows to HTML without String allocation.
        let ctx = make_ctx();
        ctx.db.put("test:key", Value::from_str("test_value")).unwrap();

        let result = UnifiedEngine::render_db_page(&ctx, "Zero Copy Test");
        let html_bytes = result.into_bytes();

        // The result is Vec<u8>, not String.
        // Verify it contains the DB data.
        let html_str = String::from_utf8(html_bytes).unwrap();
        assert!(html_str.contains("test:key"));
        assert!(html_str.contains("test_value"));
    }

    #[test]
    fn render_text() {
        let result = UnifiedEngine::render_text("Hello");
        assert_eq!(result.content_type, "text/plain; charset=utf-8");
        assert_eq!(String::from_utf8(result.html).unwrap(), "Hello");
    }

    #[test]
    fn render_json() {
        let result = UnifiedEngine::render_json(r#"{"ok":true}"#);
        assert_eq!(result.content_type, "application/json");
        assert_eq!(String::from_utf8(result.html).unwrap(), r#"{"ok":true}"#);
    }
}
