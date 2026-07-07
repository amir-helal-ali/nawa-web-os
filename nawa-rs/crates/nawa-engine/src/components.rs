//! Built-in UI components — professional, modern, zero-dependency.
//!
//! Each component writes directly to a ZeroCopyHtml buffer.
//! No String allocations, no intermediate conversions.

use crate::zerocopy_html::ZeroCopyHtml;

/// Render a navigation bar.
pub fn nav(brand: &str, links: &[(&str, &str)]) -> ZeroCopyHtml {
    let mut h = ZeroCopyHtml::new();
    h.raw_str(r#"<nav class="nawa-nav"><a class="nawa-nav-brand" href="/">"#);
    h.text(brand);
    h.raw_str(r#"</a><div class="nawa-nav-links">"#);
    for (label, href) in links {
        h.raw_str(r#"<a href="#);
        h.text(href);
        h.raw_str(r#"">"#);
        h.text(label);
        h.raw_str("</a>");
    }
    h.raw_str("</div></nav>");
    h
}

/// Render a card with a title and content.
pub fn card(title: &str, content: &str) -> ZeroCopyHtml {
    let mut h = ZeroCopyHtml::new();
    h.raw_str(r#"<div class="nawa-card"><h2>"#);
    h.text(title);
    h.raw_str("</h2><p>");
    h.text(content);
    h.raw_str("</p></div>");
    h
}

/// Button style variant.
#[derive(Debug, Clone, Copy)]
pub enum ButtonVariant {
    Primary,
    Secondary,
    Danger,
}

/// Render a button.
pub fn button(label: &str, variant: ButtonVariant) -> ZeroCopyHtml {
    let mut h = ZeroCopyHtml::new();
    let class = match variant {
        ButtonVariant::Primary => "nawa-btn nawa-btn-primary",
        ButtonVariant::Secondary => "nawa-btn nawa-btn-secondary",
        ButtonVariant::Danger => "nawa-btn nawa-btn-danger",
    };
    h.raw_str(r#"<button class="#);
    h.text(class);
    h.raw_str(r#"">"#);
    h.text(label);
    h.raw_str("</button>");
    h
}

/// Render a form input.
pub fn input(name: &str, input_type: &str, placeholder: &str) -> ZeroCopyHtml {
    let mut h = ZeroCopyHtml::new();
    h.raw_str(r#"<label class="nawa-label">"#);
    h.text(name);
    h.raw_str(r#"</label><input type="#);
    h.text(input_type);
    h.raw_str(r#"" name="#);
    h.text(name);
    h.raw_str(r#"" placeholder="#);
    h.text(placeholder);
    h.raw_str(r#"" class="nawa-input" />"#);
    h
}

/// Render a form.
pub fn form(action: &str, inputs: &[(&str, &str, &str)], button_label: &str) -> ZeroCopyHtml {
    let mut h = ZeroCopyHtml::new();
    h.raw_str(r#"<form method="POST" action="#);
    h.text(action);
    h.raw_str(r#"">"#);
    for (name, input_type, placeholder) in inputs {
        // Inline the input rendering (zero-copy, no intermediate ZeroCopyHtml).
        h.raw_str(r#"<label class="nawa-label">"#);
        h.text(name);
        h.raw_str(r#"</label><input type="#);
        h.text(input_type);
        h.raw_str(r#"" name="#);
        h.text(name);
        h.raw_str(r#"" placeholder="#);
        h.text(placeholder);
        h.raw_str(r#"" class="nawa-input" />"#);
    }
    // Inline button.
    h.raw_str(r#"<button class="nawa-btn nawa-btn-primary">"#);
    h.text(button_label);
    h.raw_str("</button></form>");
    h
}

/// Render a table from DB data (zero-copy: takes &[u8] keys and values).
pub fn db_table(headers: &[&str], rows: &[(&[u8], &[u8])]) -> ZeroCopyHtml {
    let mut h = ZeroCopyHtml::new();
    h.raw_str(r#"<table class="nawa-table"><thead><tr>"#);
    for header in headers {
        h.raw_str("<th>");
        h.text(header);
        h.raw_str("</th>");
    }
    h.raw_str("</tr></thead><tbody>");
    for (key, val) in rows {
        h.raw_str("<tr><td>");
        h.text_bytes(key);
        h.raw_str("</td><td>");
        h.text_bytes(val);
        h.raw_str("</td></tr>");
    }
    h.raw_str("</tbody></table>");
    h
}

/// Badge style variant.
#[derive(Debug, Clone, Copy)]
pub enum BadgeVariant {
    Ok,
    Warn,
    Danger,
    Info,
}

/// Render a badge.
pub fn badge(text: &str, variant: BadgeVariant) -> ZeroCopyHtml {
    let mut h = ZeroCopyHtml::new();
    let class = match variant {
        BadgeVariant::Ok => "nawa-badge nawa-badge-ok",
        BadgeVariant::Warn => "nawa-badge nawa-badge-warn",
        BadgeVariant::Danger => "nawa-badge nawa-badge-danger",
        BadgeVariant::Info => "nawa-badge nawa-badge-info",
    };
    h.raw_str(r#"<span class="#);
    h.text(class);
    h.raw_str(r#"">"#);
    h.text(text);
    h.raw_str("</span>");
    h
}

/// Render stats cards.
pub fn stats(items: &[(&str, &str)]) -> ZeroCopyHtml {
    let mut h = ZeroCopyHtml::new();
    h.raw_str(r#"<div class="nawa-stats">"#);
    for (value, label) in items {
        h.raw_str(r#"<div class="nawa-stat"><div class="nawa-stat-val">"#);
        h.text(value);
        h.raw_str(r#"</div><div class="nawa-stat-label">"#);
        h.text(label);
        h.raw_str("</div></div>");
    }
    h.raw_str("</div>");
    h
}

/// Render an island wrapper.
pub fn island(id: &str, component: &str, props_json: &str, content: &[u8]) -> ZeroCopyHtml {
    let mut h = ZeroCopyHtml::new();
    h.raw_str(r#"<div data-island="#);
    h.text(id);
    h.raw_str(r#"" data-component="#);
    h.text(component);
    h.raw_str(r#"" data-props="#);
    h.text(props_json);
    h.raw_str(r#"">"#);
    h.raw(content);
    h.raw_str("</div>");
    h
}

/// Render the hydration script for all islands.
pub fn hydration_script(islands_json: &str) -> ZeroCopyHtml {
    let mut h = ZeroCopyHtml::new();
    h.raw_str(r#"<script>window.__NAWA_ISLANDS__ = "#);
    h.raw_str(islands_json);
    h.raw_str(r#";(function(){var e=document.querySelectorAll("[data-island]");e.forEach(function(t){t.setAttribute("data-hydrated","true")})})();</script>"#);
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nav_renders() {
        let n = nav("NAWA", &[("Home", "/"), ("About", "/about")]);
        let html = String::from_utf8(n.into_bytes()).unwrap();
        assert!(html.contains("nawa-nav"));
        assert!(html.contains("Home"));
        assert!(html.contains("/about"));
    }

    #[test]
    fn card_renders() {
        let c = card("Title", "Content here");
        let html = String::from_utf8(c.into_bytes()).unwrap();
        assert!(html.contains("nawa-card"));
        assert!(html.contains("Title"));
    }

    #[test]
    fn button_variants() {
        let primary = button("Save", ButtonVariant::Primary);
        let html = String::from_utf8(primary.into_bytes()).unwrap();
        assert!(html.contains("nawa-btn-primary"));
        assert!(html.contains("Save"));
    }

    #[test]
    fn form_renders() {
        let f = form("/login", &[("email", "email", "Email"), ("password", "password", "Password")], "Login");
        let html = String::from_utf8(f.into_bytes()).unwrap();
        assert!(html.contains("<form"));
        assert!(html.contains("/login"));
        assert!(html.contains("Login"));
    }

    #[test]
    fn db_table_zero_copy() {
        let rows: Vec<(&[u8], &[u8])> = vec![
            (b"user:1", b"Ahmed"),
            (b"user:2", b"Sara"),
        ];
        let t = db_table(&["Key", "Value"], &rows);
        let html = String::from_utf8(t.into_bytes()).unwrap();
        assert!(html.contains("nawa-table"));
        assert!(html.contains("Ahmed"));
    }

    #[test]
    fn badge_renders() {
        let b = badge("Active", BadgeVariant::Ok);
        let html = String::from_utf8(b.into_bytes()).unwrap();
        assert!(html.contains("nawa-badge-ok"));
    }

    #[test]
    fn stats_renders() {
        let s = stats(&[("42", "Users"), ("99%", "Uptime")]);
        let html = String::from_utf8(s.into_bytes()).unwrap();
        assert!(html.contains("nawa-stats"));
        assert!(html.contains("42"));
    }

    #[test]
    fn island_renders() {
        let content = b"<p>Count: 0</p>";
        let i = island("counter", "Counter", r#"{"init":0}"#, content);
        let html = String::from_utf8(i.into_bytes()).unwrap();
        assert!(html.contains("data-island"));
        assert!(html.contains("data-component"));
    }

    #[test]
    fn hydration_script_renders() {
        let hs = hydration_script(r#"[{"id":"counter"}]"#);
        let html = String::from_utf8(hs.into_bytes()).unwrap();
        assert!(html.contains("__NAWA_ISLANDS__"));
        assert!(html.contains("querySelectorAll"));
    }
}
