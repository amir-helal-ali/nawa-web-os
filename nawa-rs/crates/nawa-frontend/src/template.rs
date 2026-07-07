//! Template system — reusable page layouts.
//!
//! Templates define the structure of a page (header, footer, nav)
//! and slots where content is injected.

use crate::html::HtmlNode;
use crate::island::IslandRegistry;

/// A page template — defines layout + slots.
pub struct PageTemplate {
    name: String,
    title: String,
    css: String,
    nav_items: Vec<(String, String)>, // (label, href)
    islands: IslandRegistry,
    body_content: Vec<HtmlNode>,
    /// Additional head content (scripts, meta tags).
    head_extra: Vec<HtmlNode>,
}

impl PageTemplate {
    /// Create a new template.
    pub fn new(name: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            title: title.into(),
            css: String::new(),
            nav_items: Vec::new(),
            islands: IslandRegistry::new(),
            body_content: Vec::new(),
            head_extra: Vec::new(),
        }
    }

    /// Set CSS.
    pub fn css(mut self, css: impl Into<String>) -> Self {
        self.css = css.into();
        self
    }

    /// Add a nav item.
    pub fn nav_item(mut self, label: impl Into<String>, href: impl Into<String>) -> Self {
        self.nav_items.push((label.into(), href.into()));
        self
    }

    /// Add multiple nav items.
    pub fn nav_items_list(mut self, items: Vec<(String, String)>) -> Self {
        self.nav_items = items;
        self
    }

    /// Register an island.
    pub fn island(mut self, island: crate::island::Island) -> Self {
        self.islands.register(island);
        self
    }

    /// Add body content.
    pub fn content(mut self, node: impl Into<HtmlNode>) -> Self {
        self.body_content.push(node.into());
        self
    }

    /// Add multiple body content nodes.
    pub fn content_nodes(mut self, nodes: Vec<HtmlNode>) -> Self {
        self.body_content.extend(nodes);
        self
    }

    /// Add head content (scripts, meta).
    pub fn head(mut self, node: impl Into<HtmlNode>) -> Self {
        self.head_extra.push(node.into());
        self
    }

    /// Render the complete HTML page.
    pub fn render(self) -> String {
        // Build nav.
        let nav_links: String = self.nav_items.iter()
            .map(|(label, href)| {
                format!(r#"<a href="{href}">{label}</a>"#)
            })
            .collect::<Vec<_>>()
            .join("\n");

        let nav_html = if !self.nav_items.is_empty() {
            format!(r#"<nav class="nawa-nav"><div class="nawa-nav-brand">{name}</div><div class="nawa-nav-links">{nav_links}</div></div>"#,
                name = self.name,
                nav_links = nav_links)
        } else {
            String::new()
        };

        // Build island elements.
        let island_nodes = self.islands.render_all();
        let island_html: String = island_nodes.iter()
            .map(|n| n.render())
            .collect::<Vec<_>>()
            .join("\n");

        // Build hydration script.
        let hydration_script = if !self.islands.is_empty() {
            self.islands.hydration_script()
        } else {
            String::new()
        };

        // Build body content.
        let body_html: String = self.body_content.iter()
            .map(|n| n.render())
            .collect::<Vec<_>>()
            .join("\n");

        // Build head extra.
        let head_extra_html: String = self.head_extra.iter()
            .map(|n| n.render())
            .collect::<Vec<_>>()
            .join("\n");

        // Build complete page.
        format!(
            r#"<!DOCTYPE html>
<html lang="ar" dir="rtl">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title}</title>
    <style>{css}</style>
    {head_extra}
</head>
<body>
    {nav_html}
    <main class="nawa-main">
        {body_html}
        {island_html}
    </main>
    {hydration_script}
</body>
</html>"#,
            title = self.title,
            css = self.css,
            head_extra = head_extra_html,
            nav_html = nav_html,
            body_html = body_html,
            island_html = island_html,
            hydration_script = hydration_script,
        )
    }
}

/// Builder for creating templates with fluent API.
pub struct TemplateBuilder {
    template: PageTemplate,
}

impl TemplateBuilder {
    /// Start building a new template.
    pub fn new(name: &str, title: &str) -> Self {
        Self {
            template: PageTemplate::new(name, title),
        }
    }

    /// Set CSS.
    pub fn css(mut self, css: &str) -> Self {
        self.template.css = css.to_string();
        self
    }

    /// Add nav item.
    pub fn nav(mut self, label: &str, href: &str) -> Self {
        self.template.nav_items.push((label.into(), href.into()));
        self
    }

    /// Add content.
    pub fn content(mut self, html: &str) -> Self {
        self.template.body_content.push(HtmlNode::Raw(html.into()));
        self
    }

    /// Add island.
    pub fn island(mut self, island: crate::island::Island) -> Self {
        self.template.islands.register(island);
        self
    }

    /// Build the template.
    pub fn build(self) -> PageTemplate {
        self.template
    }

    /// Build and render to HTML string.
    pub fn render(self) -> String {
        self.template.render()
    }
}

/// Default NAWA CSS theme (dark amber, RTL).
pub fn default_css() -> &'static str {
    r#"
* { margin: 0; padding: 0; box-sizing: border-box; }
body { font-family: 'Noto Sans Arabic', system-ui, sans-serif;
       background: #0d0c0a; color: #e0e0e0; line-height: 1.8;
       min-height: 100vh; }
.nawa-nav { display: flex; justify-content: space-between; align-items: center;
            padding: 1rem 2rem; background: #1a1a1a; border-bottom: 1px solid #2a2a2a; }
.nawa-nav-brand { color: #f59e0b; font-weight: bold; font-size: 1.2rem; }
.nawa-nav-links a { color: #f59e0b; text-decoration: none; margin-left: 1rem; }
.nawa-main { max-width: 900px; margin: 0 auto; padding: 2rem; }
[data-suspense] { padding: 1rem; background: #1a1a1a; border-radius: 8px;
                  border: 1px dashed #3a3a3a; color: #888; }
[data-island] { margin: 1rem 0; }
"#
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::html::h1;
    use crate::island::Island;

    #[test]
    fn basic_template() {
        let html = PageTemplate::new("NAWA", "Home")
            .content(h1().text("Hello"))
            .render();

        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<title>Home</title>"));
        assert!(html.contains("<h1>Hello</h1>"));
        assert!(html.contains("dir=\"rtl\""));
    }

    #[test]
    fn template_with_nav() {
        let html = PageTemplate::new("NAWA", "Page")
            .nav_item("Home", "/")
            .nav_item("About", "/about")
            .render();

        assert!(html.contains("nawa-nav"));
        assert!(html.contains("href=\"/\""));
        assert!(html.contains("href=\"/about\""));
    }

    #[test]
    fn template_with_island() {
        let island = Island::new("counter", "Counter")
            .content(h1().text("Count: 0"));

        let html = PageTemplate::new("NAWA", "Counter")
            .island(island)
            .render();

        assert!(html.contains("data-island=\"counter\""));
        assert!(html.contains("__NAWA_ISLANDS__"));
        assert!(html.contains("Count: 0"));
    }

    #[test]
    fn template_with_css() {
        let html = PageTemplate::new("NAWA", "Styled")
            .css("body { color: red; }")
            .render();

        assert!(html.contains("<style>body { color: red; }</style>"));
    }

    #[test]
    fn builder_pattern() {
        let html = TemplateBuilder::new("NAWA", "Built")
            .nav("Home", "/")
            .content("<p>Built with builder</p>")
            .render();

        assert!(html.contains("nawa-nav"));
        assert!(html.contains("Built with builder"));
    }

    #[test]
    fn default_css_present() {
        let css = default_css();
        assert!(css.contains("background"));
        assert!(css.contains("nawa-nav"));
    }

    #[test]
    fn template_no_islands_no_script() {
        let html = PageTemplate::new("NAWA", "No Islands")
            .content(h1().text("Static"))
            .render();

        assert!(!html.contains("__NAWA_ISLANDS__"));
    }
}
