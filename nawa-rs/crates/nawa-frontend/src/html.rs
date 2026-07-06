//! Type-safe HTML builder — Rust structs → HTML string.
//!
//! Instead of string concatenation or template engines, we use
//! typed Rust structs that serialize to HTML. This gives us:
//! - Compile-time safety (no unclosed tags)
//! - Zero allocations in hot paths (writes to a String buffer)
//! - Automatic escaping (XSS protection)

/// An HTML node — either text or an element.
#[derive(Debug, Clone)]
pub enum HtmlNode {
    /// Raw text (will be escaped).
    Text(String),
    /// An HTML element with children.
    Element(HtmlElement),
    /// Raw HTML (NOT escaped — use carefully).
    Raw(String),
    /// A comment.
    Comment(String),
}

/// An HTML element.
#[derive(Debug, Clone)]
pub struct HtmlElement {
    pub tag: String,
    pub attrs: Vec<(String, String)>,
    pub children: Vec<HtmlNode>,
    pub self_closing: bool,
}

impl HtmlElement {
    /// Create a new element.
    pub fn new(tag: impl Into<String>) -> Self {
        Self {
            tag: tag.into(),
            attrs: Vec::new(),
            children: Vec::new(),
            self_closing: false,
        }
    }

    /// Add an attribute.
    pub fn attr(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attrs.push((key.into(), value.into()));
        self
    }

    /// Add a class.
    pub fn class(mut self, class: impl Into<String>) -> Self {
        self.attrs.push(("class".into(), class.into()));
        self
    }

    /// Add an id.
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.attrs.push(("id".into(), id.into()));
        self
    }

    /// Add a child node.
    pub fn child(mut self, child: impl Into<HtmlNode>) -> Self {
        self.children.push(child.into());
        self
    }

    /// Add text child (will be escaped).
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.children.push(HtmlNode::Text(text.into()));
        self
    }

    /// Add raw HTML child (NOT escaped).
    pub fn raw(mut self, html: impl Into<String>) -> Self {
        self.children.push(HtmlNode::Raw(html.into()));
        self
    }

    /// Add multiple children.
    pub fn children(mut self, children: Vec<HtmlNode>) -> Self {
        self.children.extend(children);
        self
    }

    /// Mark as self-closing (no closing tag).
    pub fn self_closing(mut self) -> Self {
        self.self_closing = true;
        self
    }

    /// Render to a string buffer.
    pub fn render_to(&self, buf: &mut String) {
        buf.push('<');
        buf.push_str(&self.tag);
        for (k, v) in &self.attrs {
            buf.push(' ');
            buf.push_str(k);
            buf.push_str("=\"");
            buf.push_str(&escape_attr(v));
            buf.push('"');
        }
        if self.self_closing {
            buf.push_str(" />");
            return;
        }
        buf.push('>');
        for child in &self.children {
            child.render_to(buf);
        }
        buf.push_str("</");
        buf.push_str(&self.tag);
        buf.push('>');
    }

    /// Render to a string.
    pub fn render(&self) -> String {
        let mut buf = String::with_capacity(256);
        self.render_to(&mut buf);
        buf
    }
}

impl HtmlNode {
    /// Render to a string buffer.
    pub fn render_to(&self, buf: &mut String) {
        match self {
            HtmlNode::Text(t) => buf.push_str(&escape_text(t)),
            HtmlNode::Element(e) => e.render_to(buf),
            HtmlNode::Raw(r) => buf.push_str(r),
            HtmlNode::Comment(c) => {
                buf.push_str("<!-- ");
                buf.push_str(c);
                buf.push_str(" -->");
            }
        }
    }

    /// Render to a string.
    pub fn render(&self) -> String {
        let mut buf = String::with_capacity(128);
        self.render_to(&mut buf);
        buf
    }
}

impl From<&str> for HtmlNode {
    fn from(s: &str) -> Self {
        HtmlNode::Text(s.to_string())
    }
}

impl From<String> for HtmlNode {
    fn from(s: String) -> Self {
        HtmlNode::Text(s)
    }
}

impl From<HtmlElement> for HtmlNode {
    fn from(e: HtmlElement) -> Self {
        HtmlNode::Element(e)
    }
}

/// A complete HTML document.
pub struct Html {
    head: Vec<HtmlNode>,
    body: Vec<HtmlNode>,
    lang: String,
    dir: String,
    title: String,
}

impl Html {
    /// Create a new HTML document.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            head: Vec::new(),
            body: Vec::new(),
            lang: "ar".into(),
            dir: "rtl".into(),
            title: title.into(),
        }
    }

    /// Set language.
    pub fn lang(mut self, lang: impl Into<String>) -> Self {
        self.lang = lang.into();
        self
    }

    /// Set direction.
    pub fn dir(mut self, dir: impl Into<String>) -> Self {
        self.dir = dir.into();
        self
    }

    /// Add to head.
    pub fn head(mut self, node: impl Into<HtmlNode>) -> Self {
        self.head.push(node.into());
        self
    }

    /// Add a CSS style to head.
    pub fn style(self, css: impl Into<String>) -> Self {
        self.head(HtmlElement::new("style").raw(css))
    }

    /// Add a script to head.
    pub fn script(self, src: impl Into<String>) -> Self {
        self.head(HtmlElement::new("script").attr("src", src))
    }

    /// Add inline script.
    pub fn script_inline(self, js: impl Into<String>) -> Self {
        self.head(HtmlElement::new("script").raw(js))
    }

    /// Add to body.
    pub fn body(mut self, node: impl Into<HtmlNode>) -> Self {
        self.body.push(node.into());
        self
    }

    /// Add multiple body nodes.
    pub fn body_nodes(mut self, nodes: Vec<HtmlNode>) -> Self {
        self.body.extend(nodes);
        self
    }

    /// Render the complete HTML document.
    pub fn render(&self) -> String {
        let mut buf = String::with_capacity(1024);
        buf.push_str("<!DOCTYPE html>");
        buf.push_str(&format!("<html lang=\"{}\" dir=\"{}\">", self.lang, self.dir));

        // Head
        buf.push_str("<head>");
        buf.push_str("<meta charset=\"UTF-8\">");
        buf.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">");
        buf.push_str(&format!("<title>{}</title>", escape_text(&self.title)));
        for node in &self.head {
            node.render_to(&mut buf);
        }
        buf.push_str("</head>");

        // Body
        buf.push_str("<body>");
        for node in &self.body {
            node.render_to(&mut buf);
        }
        buf.push_str("</body>");

        buf.push_str("</html>");
        buf
    }
}

/// Escape text for HTML content (prevents XSS).
fn escape_text(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(c),
        }
    }
    out
}

/// Escape text for HTML attribute values.
fn escape_attr(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(c),
        }
    }
    out
}

/// Convenience functions for common elements.
pub fn div() -> HtmlElement { HtmlElement::new("div") }
pub fn span() -> HtmlElement { HtmlElement::new("span") }
pub fn h1() -> HtmlElement { HtmlElement::new("h1") }
pub fn h2() -> HtmlElement { HtmlElement::new("h2") }
pub fn h3() -> HtmlElement { HtmlElement::new("h3") }
pub fn p() -> HtmlElement { HtmlElement::new("p") }
pub fn a(href: impl Into<String>) -> HtmlElement {
    HtmlElement::new("a").attr("href", href)
}
pub fn button() -> HtmlElement { HtmlElement::new("button") }
pub fn input() -> HtmlElement { HtmlElement::new("input").self_closing() }
pub fn label() -> HtmlElement { HtmlElement::new("label") }
pub fn form(action: impl Into<String>) -> HtmlElement {
    HtmlElement::new("form").attr("action", action).attr("method", "POST")
}
pub fn nav() -> HtmlElement { HtmlElement::new("nav") }
pub fn ul() -> HtmlElement { HtmlElement::new("ul") }
pub fn li() -> HtmlElement { HtmlElement::new("li") }
pub fn table() -> HtmlElement { HtmlElement::new("table") }
pub fn tr() -> HtmlElement { HtmlElement::new("tr") }
pub fn th() -> HtmlElement { HtmlElement::new("th") }
pub fn td() -> HtmlElement { HtmlElement::new("td") }
pub fn img(src: impl Into<String>) -> HtmlElement {
    HtmlElement::new("img").attr("src", src).self_closing()
}
pub fn br() -> HtmlElement { HtmlElement::new("br").self_closing() }
pub fn hr() -> HtmlElement { HtmlElement::new("hr").self_closing() }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_element() {
        let el = h1().text("Hello");
        assert_eq!(el.render(), "<h1>Hello</h1>");
    }

    #[test]
    fn nested_elements() {
        let el = div().class("container").child(
            p().text("Hello, ").child(a("/about").text("About"))
        );
        assert_eq!(el.render(), "<div class=\"container\"><p>Hello, <a href=\"/about\">About</a></p></div>");
    }

    #[test]
    fn xss_protection() {
        let el = p().text("<script>alert('xss')</script>");
        let rendered = el.render();
        assert!(!rendered.contains("<script>"));
        assert!(rendered.contains("&lt;script&gt;"));
    }

    #[test]
    fn attr_escaping() {
        let el = a("/search?q=test\"onload=\"alert(1)");
        let rendered = el.render();
        assert!(rendered.contains("&quot;"));
        assert!(!rendered.contains("\"onload"));
    }

    #[test]
    fn self_closing() {
        let el = input().attr("type", "text").attr("name", "email");
        assert_eq!(el.render(), "<input type=\"text\" name=\"email\" />");
    }

    #[test]
    fn full_document() {
        let doc = Html::new("Test Page")
            .style("body { color: red; }")
            .body(h1().text("Hello"));
        let html = doc.render();
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<html lang=\"ar\" dir=\"rtl\">"));
        assert!(html.contains("<title>Test Page</title>"));
        assert!(html.contains("<style>body { color: red; }</style>"));
        assert!(html.contains("<h1>Hello</h1>"));
    }

    #[test]
    fn raw_html() {
        let el = div().raw("<b>bold</b>");
        assert_eq!(el.render(), "<div><b>bold</b></div>");
    }

    #[test]
    fn multiple_children() {
        let el = ul().child(li().text("A")).child(li().text("B"));
        assert_eq!(el.render(), "<ul><li>A</li><li>B</li></ul>");
    }

    #[test]
    fn comment() {
        let node = HtmlNode::Comment("note".into());
        assert_eq!(node.render(), "<!-- note -->");
    }
}
