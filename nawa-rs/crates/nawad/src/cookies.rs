//! Cookie management — secure cookie handling.
//!
//! Provides:
//! - Cookie parsing and serialization
//! - Secure cookie attributes (HttpOnly, Secure, SameSite, Max-Age)
//! - Cookie jar (per-request storage)
//! - Cookie signing (HMAC-based tamper protection)

#![allow(dead_code)]

use std::collections::HashMap;

/// A single HTTP cookie.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Cookie {
    pub name: String,
    pub value: String,
    pub domain: Option<String>,
    pub path: Option<String>,
    pub max_age: Option<u64>,
    pub http_only: bool,
    pub secure: bool,
    pub same_site: SameSite,
}

/// SameSite attribute values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SameSite {
    Strict,
    #[default]
    Lax,
    None,
}

impl Cookie {
    /// Create a new cookie.
    pub fn new(name: &str, value: &str) -> Self {
        Self {
            name: name.to_string(),
            value: value.to_string(),
            domain: None,
            path: Some("/".to_string()),
            max_age: None,
            http_only: true,
            secure: false,
            same_site: SameSite::Lax,
        }
    }

    /// Set domain.
    pub fn domain(mut self, d: &str) -> Self { self.domain = Some(d.into()); self }

    /// Set path.
    pub fn path(mut self, p: &str) -> Self { self.path = Some(p.into()); self }

    /// Set max age (seconds).
    pub fn max_age(mut self, secs: u64) -> Self { self.max_age = Some(secs); self }

    /// Set HttpOnly.
    pub fn http_only(mut self, b: bool) -> Self { self.http_only = b; self }

    /// Set Secure.
    pub fn secure(mut self, b: bool) -> Self { self.secure = b; self }

    /// Set SameSite.
    pub fn same_site(mut self, s: SameSite) -> Self { self.same_site = s; self }

    /// Serialize to Set-Cookie header value.
    pub fn to_header(&self) -> String {
        let mut h = format!("{}={}", self.name, self.value);
        if let Some(d) = &self.domain { h.push_str(&format!("; Domain={d}")); }
        if let Some(p) = &self.path { h.push_str(&format!("; Path={p}")); }
        if let Some(a) = self.max_age { h.push_str(&format!("; Max-Age={a}")); }
        if self.http_only { h.push_str("; HttpOnly"); }
        if self.secure { h.push_str("; Secure"); }
        h.push_str(&format!("; SameSite={}", match self.same_site {
            SameSite::Strict => "Strict",
            SameSite::Lax => "Lax",
            SameSite::None => "None",
        }));
        h
    }

    /// Parse cookies from a Cookie header value.
    pub fn parse(header: &str) -> HashMap<String, String> {
        let mut cookies = HashMap::new();
        for pair in header.split(';') {
            let pair = pair.trim();
            if let Some((k, v)) = pair.split_once('=') {
                cookies.insert(k.trim().to_string(), v.trim().to_string());
            }
        }
        cookies
    }

    /// Get a cookie value by name from a Cookie header.
    pub fn get(header: &str, name: &str) -> Option<String> {
        Self::parse(header).get(name).cloned()
    }

    /// Create a signed cookie (HMAC-SHA256).
    pub fn sign(value: &str, secret: &str) -> String {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;
        type HmacSha256 = Hmac<Sha256>;
        if let Ok(mut mac) = HmacSha256::new_from_slice(secret.as_bytes()) {
            mac.update(value.as_bytes());
            let sig = hex::encode(mac.finalize().into_bytes());
            format!("{value}.{sig}")
        } else {
            value.to_string()
        }
    }

    /// Verify a signed cookie.
    pub fn verify(signed: &str, secret: &str) -> Option<String> {
        let (value, sig) = signed.split_once('.')?;
        let expected = Self::sign(value, secret);
        let (_, expected_sig) = expected.split_once('.')?;
        if sig == expected_sig {
            Some(value.to_string())
        } else {
            None
        }
    }

    /// Create an expired cookie (for deletion).
    pub fn expired(name: &str) -> Self {
        Self {
            name: name.to_string(),
            value: String::new(),
            domain: None,
            path: Some("/".into()),
            max_age: Some(0),
            http_only: true,
            secure: false,
            same_site: SameSite::Lax,
        }
    }
}

/// Cookie jar — stores cookies for a response.
#[derive(Debug, Clone, Default)]
pub struct CookieJar {
    cookies: Vec<Cookie>,
}

impl CookieJar {
    pub fn new() -> Self { Self::default() }

    pub fn add(&mut self, cookie: Cookie) { self.cookies.push(cookie); }

    pub fn remove(&mut self, name: &str) {
        self.cookies.push(Cookie::expired(name));
    }

    pub fn to_headers(&self) -> Vec<(String, String)> {
        self.cookies.iter()
            .map(|c| ("Set-Cookie".into(), c.to_header()))
            .collect()
    }

    pub fn len(&self) -> usize { self.cookies.len() }
    pub fn is_empty(&self) -> bool { self.cookies.is_empty() }
}

/// CORS configuration.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CorsConfig {
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<String>,
    pub allowed_headers: Vec<String>,
    pub exposed_headers: Vec<String>,
    pub allow_credentials: bool,
    pub max_age: u64,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            allowed_origins: vec!["*".into()],
            allowed_methods: vec!["GET".into(), "POST".into(), "PUT".into(), "DELETE".into(), "OPTIONS".into()],
            allowed_headers: vec!["Content-Type".into(), "Authorization".into(), "Cookie".into(), "X-API-Version".into(), "X-CSRF-Token".into()],
            exposed_headers: vec!["X-Response-Time".into(), "X-Powered-By".into()],
            allow_credentials: false,
            max_age: 3600,
        }
    }
}

impl CorsConfig {
    /// Create a strict CORS config (same-origin only).
    pub fn strict() -> Self {
        Self {
            allowed_origins: vec![], // empty = same origin only
            allow_credentials: true,
            ..Default::default()
        }
    }

    /// Create a permissive CORS config (all origins).
    pub fn permissive() -> Self {
        Self::default()
    }

    /// Check if an origin is allowed.
    pub fn is_origin_allowed(&self, origin: &str) -> bool {
        if self.allowed_origins.is_empty() { return false; }
        if self.allowed_origins.contains(&"*".to_string()) { return true; }
        self.allowed_origins.iter().any(|o| o == origin)
    }

    /// Generate CORS headers for a response.
    pub fn headers(&self, origin: Option<&str>) -> Vec<(String, String)> {
        let mut headers = Vec::new();
        if let Some(o) = origin {
            if self.is_origin_allowed(o) {
                headers.push(("Access-Control-Allow-Origin".into(), o.to_string()));
            } else if self.allowed_origins.contains(&"*".to_string()) {
                headers.push(("Access-Control-Allow-Origin".into(), "*".into()));
            }
        }
        headers.push(("Access-Control-Allow-Methods".into(), self.allowed_methods.join(", ")));
        headers.push(("Access-Control-Allow-Headers".into(), self.allowed_headers.join(", ")));
        if !self.exposed_headers.is_empty() {
            headers.push(("Access-Control-Expose-Headers".into(), self.exposed_headers.join(", ")));
        }
        if self.allow_credentials {
            headers.push(("Access-Control-Allow-Credentials".into(), "true".into()));
        }
        headers.push(("Access-Control-Max-Age".into(), self.max_age.to_string()));
        headers
    }

    /// Check if a preflight request should be handled.
    pub fn is_preflight(method: &str, headers: &HashMap<String, String>) -> bool {
        method == "OPTIONS" && headers.contains_key("access-control-request-method")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cookie_to_header_includes_attributes() {
        let c = Cookie::new("token", "abc123")
            .max_age(3600)
            .http_only(true)
            .secure(true)
            .same_site(SameSite::Strict);
        let h = c.to_header();
        assert!(h.contains("token=abc123"));
        assert!(h.contains("Max-Age=3600"));
        assert!(h.contains("HttpOnly"));
        assert!(h.contains("Secure"));
        assert!(h.contains("SameSite=Strict"));
    }

    #[test]
    fn cookie_parse_extracts_pairs() {
        let cookies = Cookie::parse("name=value; token=abc; session=xyz");
        assert_eq!(cookies.get("name"), Some(&"value".to_string()));
        assert_eq!(cookies.get("token"), Some(&"abc".to_string()));
        assert_eq!(cookies.get("session"), Some(&"xyz".to_string()));
    }

    #[test]
    fn cookie_get_by_name() {
        let header = "nawa_token=eyJhbG; other=foo";
        assert_eq!(Cookie::get(header, "nawa_token"), Some("eyJhbG".into()));
        assert_eq!(Cookie::get(header, "nonexistent"), None);
    }

    #[test]
    fn cookie_sign_and_verify() {
        let signed = Cookie::sign("secret_value", "my_secret");
        assert!(signed.contains('.'));
        let verified = Cookie::verify(&signed, "my_secret");
        assert_eq!(verified, Some("secret_value".into()));
    }

    #[test]
    fn cookie_verify_wrong_secret_fails() {
        let signed = Cookie::sign("value", "correct_secret");
        assert_eq!(Cookie::verify(&signed, "wrong_secret"), None);
    }

    #[test]
    fn cookie_expired_has_max_age_zero() {
        let c = Cookie::expired("token");
        let h = c.to_header();
        assert!(h.contains("Max-Age=0"));
    }

    #[test]
    fn cookie_jar_add_and_remove() {
        let mut jar = CookieJar::new();
        jar.add(Cookie::new("token", "abc"));
        assert_eq!(jar.len(), 1);
        jar.remove("token");
        assert_eq!(jar.len(), 2); // original + expired
    }

    #[test]
    fn cookie_jar_to_headers() {
        let mut jar = CookieJar::new();
        jar.add(Cookie::new("a", "1"));
        jar.add(Cookie::new("b", "2"));
        let headers = jar.to_headers();
        assert_eq!(headers.len(), 2);
        assert_eq!(headers[0].0, "Set-Cookie");
    }

    #[test]
    fn cors_default_allows_all_origins() {
        let cors = CorsConfig::default();
        assert!(cors.is_origin_allowed("https://example.com"));
        assert!(cors.is_origin_allowed("https://anything.com"));
    }

    #[test]
    fn cors_strict_blocks_foreign_origins() {
        let cors = CorsConfig::strict();
        assert!(!cors.is_origin_allowed("https://evil.com"));
    }

    #[test]
    fn cors_headers_include_methods() {
        let cors = CorsConfig::default();
        let headers = cors.headers(Some("https://example.com"));
        let methods = headers.iter().find(|(k, _)| k == "Access-Control-Allow-Methods");
        assert!(methods.is_some());
        assert!(methods.unwrap().1.contains("GET"));
        assert!(methods.unwrap().1.contains("POST"));
    }

    #[test]
    fn cors_is_preflight_detects_options() {
        let mut headers = HashMap::new();
        headers.insert("access-control-request-method".into(), "POST".into());
        assert!(CorsConfig::is_preflight("OPTIONS", &headers));
        assert!(!CorsConfig::is_preflight("GET", &headers));
    }

    #[test]
    fn cors_with_credentials() {
        let cors = CorsConfig { allow_credentials: true, ..Default::default() };
        let headers = cors.headers(Some("https://example.com"));
        assert!(headers.iter().any(|(k, v)| k == "Access-Control-Allow-Credentials" && v == "true"));
    }
}
