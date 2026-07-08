//! NAWA WASM SSR Demo Module
//!
//! This module is compiled to WASM and loaded by NAWA's `SsrModule`.
//! It exposes:
//! - `memory` (exported automatically by WASM)
//! - `alloc(size: i32) -> i32` — allocate a buffer in linear memory
//! - `render(props_ptr: i32, props_len: i32) -> i32` — render HTML from JSON props
//!
//! The `render` function:
//! 1. Reads JSON props from the given memory pointer.
//! 2. Parses them into a `PageProps` struct.
//! 3. Renders an HTML page with the props.
//! 4. Writes the HTML to a buffer and returns its pointer (null-terminated).
//!
//! ## Props format (JSON)
//!
//! ```json
//! {
//!   "title": "Hello NAWA",
//!   "description": "My first SSR page",
//!   "items": ["item 1", "item 2", "item 3"],
//!   "user": { "name": "Ahmed", "role": "admin" }
//! }
//! ```

// Allow static_mut_refs — WASM is single-threaded, so mutable statics are safe.
#![allow(static_mut_refs)]

use serde::Deserialize;

/// Props passed from NAWA to the WASM module.
#[derive(Debug, Deserialize, Default)]
struct PageProps {
    title: String,
    description: String,
    items: Vec<String>,
    user: Option<UserInfo>,
}

/// User info embedded in props.
#[derive(Debug, Deserialize)]
struct UserInfo {
    name: String,
    role: String,
}

/// Size of the output buffer (64 KiB).
const OUTPUT_BUFFER_SIZE: usize = 65536;

/// Static buffer for the rendered HTML output.
/// We use a static mutable buffer because WASM is single-threaded.
static mut OUTPUT_BUFFER: [u8; OUTPUT_BUFFER_SIZE] = [0u8; OUTPUT_BUFFER_SIZE];

/// Allocate `size` bytes in linear memory.
/// Returns a pointer to the allocated buffer.
/// For simplicity, we use a simple bump allocator that never frees.
/// This is fine for SSR — each render allocates once.
#[no_mangle]
pub extern "C" fn alloc(size: i32) -> i32 {
    // Allocate `size` bytes using Rust's allocator.
    // The pointer is valid for the lifetime of the WASM instance.
    let size = size as usize;
    let mut buf = Vec::with_capacity(size);
    // Ensure the vector has the right length so the pointer is valid.
    buf.resize(size, 0);
    // Forget the vector so its buffer isn't freed — the caller owns it now.
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr as i32
}

/// Render HTML from JSON props.
///
/// - `props_ptr`: pointer to JSON props in linear memory
/// - `props_len`: length of the JSON props
/// - Returns: pointer to null-terminated HTML string in linear memory
#[no_mangle]
pub extern "C" fn render(props_ptr: i32, props_len: i32) -> i32 {
    let props_ptr = props_ptr as *const u8;
    let props_len = props_len as usize;

    // Read the JSON props from memory.
    let props_bytes = unsafe {
        std::slice::from_raw_parts(props_ptr, props_len)
    };

    // Parse the JSON props.
    let props: PageProps = match serde_json::from_slice(props_bytes) {
        Ok(p) => p,
        Err(_) => {
            // On parse error, render an error page.
            let error_html = "<html><body><h1>SSR Error</h1><p>Invalid props JSON</p></body></html>";
            return write_output(error_html);
        }
    };

    // Render the HTML.
    let html = render_html(&props);
    write_output(&html)
}

/// Write a string to the output buffer and return its pointer.
/// The string is null-terminated.
///
/// SAFETY: This is safe because WASM is single-threaded, so there's no
/// data race when accessing OUTPUT_BUFFER.
fn write_output(s: &str) -> i32 {
    // Use raw pointer operations to avoid `static_mut_refs` warning.
    let bytes = s.as_bytes();
    let len = bytes.len();
    unsafe {
        let buf_ptr = OUTPUT_BUFFER.as_mut_ptr();
        let buf_slice = std::slice::from_raw_parts_mut(buf_ptr, OUTPUT_BUFFER_SIZE);
        // Check if it fits in the buffer.
        if len + 1 > OUTPUT_BUFFER_SIZE {
            // Truncate if too large.
            let truncated = &bytes[..OUTPUT_BUFFER_SIZE - 1];
            buf_slice[..truncated.len()].copy_from_slice(truncated);
            buf_slice[truncated.len()] = 0;
        } else {
            buf_slice[..len].copy_from_slice(bytes);
            buf_slice[len] = 0; // null terminator
        }
        buf_ptr as i32
    }
}

/// Render the HTML page from props.
fn render_html(props: &PageProps) -> String {
    let mut html = String::with_capacity(4096);

    // HTML head.
    html.push_str("<!DOCTYPE html>\n<html lang=\"ar\" dir=\"rtl\">\n<head>\n");
    html.push_str("<meta charset=\"UTF-8\">\n");
    html.push_str("<meta name=\"viewport\" content=\"width=device-width,initial-scale=1.0\">\n");
    html.push_str(&format!("<title>{}</title>\n", html_escape(&props.title)));
    if !props.description.is_empty() {
        html.push_str(&format!(
            "<meta name=\"description\" content=\"{}\">\n",
            html_escape(&props.description)
        ));
    }
    // Inline CSS for a nice-looking page.
    html.push_str("<style>\n");
    html.push_str("body{font-family:'Noto Sans Arabic',system-ui,sans-serif;background:#0d0c0a;color:#e0e0e0;line-height:1.8;max-width:800px;margin:2rem auto;padding:0 1rem}\n");
    html.push_str("h1{color:#f59e0b}\n");
    html.push_str(".card{background:#1a1a1a;border:1px solid #2a2a2a;border-radius:12px;padding:1.5rem;margin:1rem 0}\n");
    html.push_str(".user{color:#10b981;font-weight:bold}\n");
    html.push_str("ul{padding-right:1.5rem}\n");
    html.push_str("li{margin-bottom:0.5rem}\n");
    html.push_str("</style>\n");
    html.push_str("</head>\n<body>\n");

    // HTML body.
    html.push_str(&format!("<h1>{}</h1>\n", html_escape(&props.title)));
    if !props.description.is_empty() {
        html.push_str(&format!("<p>{}</p>\n", html_escape(&props.description)));
    }

    // User info (if present).
    if let Some(user) = &props.user {
        html.push_str("<div class=\"card\">\n");
        html.push_str("<h2>المستخدم</h2>\n");
        html.push_str(&format!(
            "<p>الاسم: <span class=\"user\">{}</span></p>\n",
            html_escape(&user.name)
        ));
        html.push_str(&format!("<p>الدور: {}</p>\n", html_escape(&user.role)));
        html.push_str("</div>\n");
    }

    // Items list (if present).
    if !props.items.is_empty() {
        html.push_str("<div class=\"card\">\n");
        html.push_str("<h2>العناصر</h2>\n");
        html.push_str("<ul>\n");
        for item in &props.items {
            html.push_str(&format!("<li>{}</li>\n", html_escape(item)));
        }
        html.push_str("</ul>\n");
        html.push_str("</div>\n");
    }

    // Footer.
    html.push_str("<footer>\n<p>Rendered by NAWA WASM SSR (Rust compiled to WebAssembly)</p>\n</footer>\n");
    html.push_str("</body>\n</html>");

    html
}

/// Escape HTML special characters.
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
