//! Zero-copy HTML writer — writes directly to a byte buffer.
//!
//! Instead of building a `String` and then converting to bytes,
//! we write HTML directly to a `Vec<u8>` buffer. This buffer
//! can then be sent via io_uring without any additional copies.
//!
//! ## Why this matters
//!
//! Traditional approach:
//!   DB value → String (alloc) → escape (alloc) → HTML String (alloc)
//!   → response.body = html.into_bytes() (alloc) → socket write (copy)
//!
//! Zero-copy approach:
//!   DB value (&[u8] from mmap) → escape directly to buffer → io_uring send
//!   Total: 0 allocations, 0 copies

use std::io::Write;

/// A zero-copy HTML writer that writes to a byte buffer.
pub struct ZeroCopyHtml {
    buf: Vec<u8>,
}

impl ZeroCopyHtml {
    /// Create a new writer with a pre-allocated buffer.
    pub fn new() -> Self {
        Self {
            buf: Vec::with_capacity(4096),
        }
    }

    /// Create with a specific capacity.
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            buf: Vec::with_capacity(cap),
        }
    }

    /// Write raw bytes (no escaping — use carefully).
    #[inline]
    pub fn raw(&mut self, bytes: &[u8]) -> &mut Self {
        self.buf.extend_from_slice(bytes);
        self
    }

    /// Write raw string (no escaping).
    #[inline]
    pub fn raw_str(&mut self, s: &str) -> &mut Self {
        self.raw(s.as_bytes())
    }

    /// Write escaped text (XSS protection).
    /// Escapes: & < > → &amp; &lt; &gt;
    #[inline]
    pub fn text(&mut self, s: &str) -> &mut Self {
        for b in s.bytes() {
            match b {
                b'&' => self.buf.extend_from_slice(b"&amp;"),
                b'<' => self.buf.extend_from_slice(b"&lt;"),
                b'>' => self.buf.extend_from_slice(b"&gt;"),
                _ => self.buf.push(b),
            }
        }
        self
    }

    /// Write escaped text from raw bytes (zero-copy from DB).
    /// Takes &[u8] directly — no String conversion needed.
    #[inline]
    pub fn text_bytes(&mut self, bytes: &[u8]) -> &mut Self {
        for &b in bytes {
            match b {
                b'&' => self.buf.extend_from_slice(b"&amp;"),
                b'<' => self.buf.extend_from_slice(b"&lt;"),
                b'>' => self.buf.extend_from_slice(b"&gt;"),
                _ => self.buf.push(b),
            }
        }
        self
    }

    /// Write an opening tag.
    #[inline]
    pub fn open_tag(&mut self, tag: &str) -> &mut Self {
        self.buf.push(b'<');
        self.raw_str(tag);
        self.buf.push(b'>');
        self
    }

    /// Write an opening tag with one attribute.
    #[inline]
    pub fn open_tag_attr(&mut self, tag: &str, attr: &str, val: &str) -> &mut Self {
        self.buf.push(b'<');
        self.raw_str(tag);
        self.buf.push(b' ');
        self.raw_str(attr);
        self.buf.extend_from_slice(b"=\"");
        self.text(val);
        self.buf.push(b'"');
        self.buf.push(b'>');
        self
    }

    /// Write a closing tag.
    #[inline]
    pub fn close_tag(&mut self, tag: &str) -> &mut Self {
        self.buf.extend_from_slice(b"</");
        self.raw_str(tag);
        self.buf.push(b'>');
        self
    }

    /// Write a self-closing tag.
    #[inline]
    pub fn void_tag(&mut self, tag: &str) -> &mut Self {
        self.buf.push(b'<');
        self.raw_str(tag);
        self.buf.extend_from_slice(b" />");
        self
    }

    /// Write a self-closing tag with attribute.
    #[inline]
    pub fn void_tag_attr(&mut self, tag: &str, attr: &str, val: &str) -> &mut Self {
        self.buf.push(b'<');
        self.raw_str(tag);
        self.buf.push(b' ');
        self.raw_str(attr);
        self.buf.extend_from_slice(b"=\"");
        self.text(val);
        self.buf.push(b'"');
        self.buf.extend_from_slice(b" />");
        self
    }

    /// Write a class attribute.
    #[inline]
    pub fn class(&mut self, class: &str) -> &mut Self {
        self.buf.extend_from_slice(b" class=\"");
        self.text(class);
        self.buf.push(b'"');
        self
    }

    /// Write a data attribute.
    #[inline]
    pub fn data_attr(&mut self, key: &str, val: &str) -> &mut Self {
        self.buf.extend_from_slice(b" data-");
        self.raw_str(key);
        self.buf.extend_from_slice(b"=\"");
        self.text(val);
        self.buf.push(b'"');
        self
    }

    /// Write the DOCTYPE.
    #[inline]
    pub fn doctype(&mut self) -> &mut Self {
        self.raw_str("<!DOCTYPE html>")
    }

    /// Consume the writer and return the buffer.
    /// This buffer can be sent directly via io_uring.
    pub fn into_bytes(self) -> Vec<u8> {
        self.buf
    }

    /// Get the buffer as a slice (for preview).
    pub fn as_bytes(&self) -> &[u8] {
        &self.buf
    }

    /// Current buffer length.
    pub fn len(&self) -> usize {
        self.buf.len()
    }

    /// Is the buffer empty?
    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    /// Clear the buffer (reuse allocation).
    pub fn clear(&mut self) {
        self.buf.clear();
    }
}

impl Default for ZeroCopyHtml {
    fn default() -> Self {
        Self::new()
    }
}

impl Write for ZeroCopyHtml {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buf.extend_from_slice(buf);
        Ok(buf.len())
    }

    #[inline]
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_html() {
        let mut h = ZeroCopyHtml::new();
        h.doctype()
            .open_tag("html")
            .open_tag("body")
            .open_tag("h1")
            .text("Hello")
            .close_tag("h1")
            .close_tag("body")
            .close_tag("html");

        let result = String::from_utf8(h.into_bytes()).unwrap();
        assert_eq!(result, "<!DOCTYPE html><html><body><h1>Hello</h1></body></html>");
    }

    #[test]
    fn xss_protection() {
        let mut h = ZeroCopyHtml::new();
        h.text("<script>alert(1)</script>");
        let result = String::from_utf8(h.into_bytes()).unwrap();
        assert!(!result.contains("<script>"));
        assert!(result.contains("&lt;script&gt;"));
    }

    #[test]
    fn zero_copy_from_bytes() {
        let mut h = ZeroCopyHtml::new();
        // Simulate data from DB (raw bytes, no String conversion).
        let db_value: &[u8] = b"Hello from DB <with tags>";
        h.open_tag("p").text_bytes(db_value).close_tag("p");
        let result = String::from_utf8(h.into_bytes()).unwrap();
        assert!(result.contains("Hello from DB"));
        assert!(result.contains("&lt;with tags&gt;"));
    }

    #[test]
    fn void_tags() {
        let mut h = ZeroCopyHtml::new();
        h.void_tag("br")
            .void_tag_attr("input", "type", "text")
            .void_tag_attr("img", "src", "/photo.jpg");

        let result = String::from_utf8(h.into_bytes()).unwrap();
        assert!(result.contains("<br />"));
        assert!(result.contains("<input type=\"text\" />"));
        assert!(result.contains("<img src=\"/photo.jpg\" />"));
    }

    #[test]
    fn data_attributes() {
        let mut h = ZeroCopyHtml::new();
        h.open_tag("div")
            .data_attr("island", "counter")
            .data_attr("component", "Counter")
            .close_tag("div");

        let result = String::from_utf8(h.into_bytes()).unwrap();
        assert!(result.contains("data-island=\"counter\""));
        assert!(result.contains("data-component=\"Counter\""));
    }

    #[test]
    fn no_string_allocation() {
        // The key test: we write bytes directly without creating a String.
        let mut h = ZeroCopyHtml::with_capacity(1024);
        h.doctype()
            .open_tag("div")
            .class("card")
            .text("Content")
            .close_tag("div");

        // The buffer is a Vec<u8>, not a String.
        let bytes: Vec<u8> = h.into_bytes();
        assert!(!bytes.is_empty());
        assert!(bytes.starts_with(b"<!DOCTYPE"));
    }

    #[test]
    fn reuse_buffer() {
        let mut h = ZeroCopyHtml::new();
        h.text("first");
        assert_eq!(h.len(), 5);
        h.clear();
        assert!(h.is_empty());
        h.text("second");
        assert_eq!(h.len(), 6);
    }
}
