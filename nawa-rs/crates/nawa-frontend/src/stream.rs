//! Streaming SSR with Suspense — progressive page rendering.
//!
//! Instead of waiting for all data before sending the page, we:
//! 1. Send the HTML skeleton immediately (header, nav, static content).
//! 2. Send placeholder `<div data-suspense="...">` for slow data.
//! 3. As data becomes available, stream `<script>` chunks that replace placeholders.
//! 4. Client sees content appear progressively.

use crate::html::{Html, HtmlElement, HtmlNode};
use std::time::Duration;

/// A Suspense boundary — wraps content that may not be ready yet.
pub struct Suspense {
    /// Unique ID for this suspense boundary.
    pub id: String,
    /// Fallback content (shown while waiting).
    pub fallback: Vec<HtmlNode>,
    /// The async content (rendered when ready).
    pub content: Option<Vec<HtmlNode>>,
    /// Is the content ready?
    pub ready: bool,
}

impl Suspense {
    /// Create a new Suspense boundary.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            fallback: vec![HtmlNode::Text("Loading...".into())],
            content: None,
            ready: false,
        }
    }

    /// Set the fallback content.
    pub fn fallback(mut self, node: impl Into<HtmlNode>) -> Self {
        self.fallback = vec![node.into()];
        self
    }

    /// Set the actual content (marks as ready).
    pub fn resolve(mut self, nodes: Vec<HtmlNode>) -> Self {
        self.content = Some(nodes);
        self.ready = true;
        self
    }

    /// Render to HTML.
    /// If ready: render the content directly.
    /// If not ready: render a placeholder div with fallback.
    pub fn render(&self) -> String {
        if self.ready {
            if let Some(content) = &self.content {
                let mut buf = String::new();
                for node in content {
                    node.render_to(&mut buf);
                }
                return buf;
            }
        }

        // Not ready — render placeholder.
        let mut el = HtmlElement::new("div")
            .attr("data-suspense", &self.id)
            .attr("data-status", "pending");

        for node in &self.fallback {
            el = el.child(node.clone());
        }

        el.render()
    }

    /// Generate a streaming chunk to replace the placeholder.
    ///
    /// This is sent as a `<script>` tag that replaces the
    /// placeholder div with the actual content.
    pub fn render_stream_chunk(&self) -> Option<String> {
        if !self.ready {
            return None;
        }

        let content = self.content.as_ref()?;
        let mut html = String::new();
        for node in content {
            node.render_to(&mut html);
        }

        // Escape for JS string.
        let escaped = html.replace('\\', "\\\\").replace('\'', "\\'").replace('\n', "\\n");

        Some(format!(
            r#"<script>
(function() {{
    var el = document.querySelector('[data-suspense="{}"]');
    if (el) {{
        el.removeAttribute('data-status');
        el.setAttribute('data-status', 'resolved');
        el.innerHTML = '{}';
    }}
}})();
</script>"#,
            self.id, escaped
        ))
    }
}

/// Result of a streaming render.
pub struct SuspenseResult {
    /// The initial HTML chunk (sent immediately).
    pub initial_html: String,
    /// Streaming chunks (sent as data becomes available).
    pub chunks: Vec<String>,
}

/// A streaming response — progressively sends HTML chunks.
pub struct StreamingResponse {
    /// The HTML document template (with suspense placeholders).
    pub template: Html,
    /// Suspense boundaries in this response.
    pub suspense: Vec<Suspense>,
    /// Simulated delay before each suspense resolves (for testing).
    pub delays: Vec<Duration>,
}

impl StreamingResponse {
    /// Create a new streaming response.
    pub fn new(template: Html) -> Self {
        Self {
            template,
            suspense: Vec::new(),
            delays: Vec::new(),
        }
    }

    /// Add a Suspense boundary.
    pub fn suspense(mut self, s: Suspense) -> Self {
        self.suspense.push(s);
        self
    }

    /// Add a Suspense with a simulated delay (for testing/demo).
    pub fn suspense_with_delay(mut self, s: Suspense, delay: Duration) -> Self {
        self.suspense.push(s);
        self.delays.push(delay);
        self
    }

    /// Render the response.
    ///
    /// In a real server, this would:
    /// 1. Send `initial_html` immediately.
    /// 2. For each suspense, await its data, then send `chunk`.
    /// 3. Send closing `</body></html>` when all suspense resolved.
    pub fn render(self) -> SuspenseResult {
        // Build initial HTML with suspense placeholders.
        let mut template = self.template;

        // Add suspense placeholders to body.
        for s in &self.suspense {
            template = template.body(HtmlNode::Raw(s.render()));
        }

        let initial_html = template.render();

        // Collect streaming chunks (for resolved suspense).
        let mut chunks = Vec::new();
        for s in &self.suspense {
            if let Some(chunk) = s.render_stream_chunk() {
                chunks.push(chunk);
            }
        }

        // Add closing chunk.
        chunks.push("</body></html>".into());

        SuspenseResult {
            initial_html,
            chunks,
        }
    }

    /// Simulate async streaming (returns chunks with delays).
    pub async fn stream_chunks(&self) -> Vec<(Duration, String)> {
        let mut result = Vec::new();

        for (i, s) in self.suspense.iter().enumerate() {
            let delay = self.delays.get(i).copied().unwrap_or(Duration::from_millis(100));
            if let Some(chunk) = s.render_stream_chunk() {
                tokio::time::sleep(delay).await;
                result.push((delay, chunk));
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::html::{div, h1, p};

    #[test]
    fn suspense_not_ready() {
        let s = Suspense::new("data-1").fallback(p().text("Loading..."));
        let html = s.render();
        assert!(html.contains("data-suspense=\"data-1\""));
        assert!(html.contains("data-status=\"pending\""));
        assert!(html.contains("Loading..."));
    }

    #[test]
    fn suspense_resolved() {
        let s = Suspense::new("data-1")
            .fallback(p().text("Loading..."))
            .resolve(vec![h1().text("Data loaded!").into()]);

        let html = s.render();
        assert!(html.contains("<h1>Data loaded!</h1>"));
        assert!(!html.contains("data-suspense"));
    }

    #[test]
    fn suspense_stream_chunk() {
        let s = Suspense::new("data-1")
            .resolve(vec![p().text("Resolved content").into()]);

        let chunk = s.render_stream_chunk().unwrap();
        assert!(chunk.contains("<script>"));
        assert!(chunk.contains("data-suspense"));
        assert!(chunk.contains("Resolved content"));
        assert!(chunk.contains("innerHTML"));
    }

    #[test]
    fn streaming_response() {
        let template = Html::new("Test")
            .body(h1().text("Page Title"));

        // Create an UNRESOLVED suspense (not ready) for initial render.
        let s1 = Suspense::new("list")
            .fallback(p().text("Loading list..."));

        let response = StreamingResponse::new(template)
            .suspense(s1);

        let result = response.render();

        // Initial HTML should have the page structure + suspense placeholder.
        assert!(result.initial_html.contains("<!DOCTYPE html>"));
        assert!(result.initial_html.contains("Page Title"));
        assert!(result.initial_html.contains("data-suspense"));
        assert!(result.initial_html.contains("data-status=\"pending\""));
    }

    #[tokio::test]
    async fn async_streaming() {
        let s = Suspense::new("async-data")
            .resolve(vec![p().text("Async data").into()]);

        let response = StreamingResponse::new(Html::new("Async"))
            .suspense_with_delay(s, Duration::from_millis(50));

        let chunks = response.stream_chunks().await;
        assert_eq!(chunks.len(), 1);
        assert!(chunks[0].1.contains("Async data"));
    }

    #[test]
    fn multiple_suspense() {
        let template = Html::new("Multi")
            .body(div().text("Static content"));

        let s1 = Suspense::new("a").fallback(p().text("Loading A"));
        let s2 = Suspense::new("b").fallback(p().text("Loading B"));

        let response = StreamingResponse::new(template)
            .suspense(s1)
            .suspense(s2);

        let result = response.render();
        assert!(result.initial_html.contains("data-suspense=\"a\""));
        assert!(result.initial_html.contains("data-suspense=\"b\""));
    }
}
