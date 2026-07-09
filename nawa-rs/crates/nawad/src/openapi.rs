//! OpenAPI/Swagger auto-documentation.
//!
//! Generates an OpenAPI 3.0 spec from the registered routes.
//! Serves a Swagger UI page at /docs.

#![allow(dead_code)]

use std::collections::BTreeMap;

/// OpenAPI 3.0 spec.
#[derive(Debug, Clone, serde::Serialize)]
pub struct OpenApiSpec {
    pub openapi: String,
    pub info: OpenApiInfo,
    pub paths: BTreeMap<String, PathItem>,
    pub components: Components,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct OpenApiInfo {
    pub title: String,
    pub description: String,
    pub version: String,
    pub license: Option<OpenApiLicense>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct OpenApiLicense {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PathItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub get: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delete: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub put: Option<Operation>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Operation {
    pub summary: String,
    pub description: String,
    pub tags: Vec<String>,
    pub responses: BTreeMap<String, Response>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<Vec<BTreeMap<String, Vec<String>>>>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Response {
    pub description: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Components {
    pub security_schemes: BTreeMap<String, SecurityScheme>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SecurityScheme {
    #[serde(rename = "type")]
    pub scheme_type: String,
    pub scheme: String,
    pub bearer_format: String,
}

/// Build the OpenAPI spec from the endpoint list.
pub fn build_spec(endpoints: &[&str]) -> OpenApiSpec {
    let mut paths: BTreeMap<String, PathItem> = BTreeMap::new();

    for endpoint in endpoints {
        let parts: Vec<&str> = endpoint.splitn(2, ' ').collect();
        if parts.len() != 2 {
            continue;
        }
        let method = parts[0];
        let path = parts[1];

        // Convert NAWA path patterns to OpenAPI format.
        let openapi_path = convert_path(path);

        let path_item = paths.entry(openapi_path).or_insert_with(|| PathItem {
            get: None,
            post: None,
            delete: None,
            put: None,
        });

        let (tag, summary, description) = categorize_endpoint(path);

        let operation = Operation {
            summary: summary.clone(),
            description,
            tags: vec![tag],
            responses: build_responses(path),
            security: if needs_auth(path) {
                Some(vec![BTreeMap::from([("bearerAuth".to_string(), vec![])])])
            } else {
                None
            },
        };

        match method {
            "GET" => path_item.get = Some(operation),
            "POST" => path_item.post = Some(operation),
            "DELETE" => path_item.delete = Some(operation),
            "PUT" => path_item.put = Some(operation),
            _ => {}
        }
    }

    OpenApiSpec {
        openapi: "3.0.3".into(),
        info: OpenApiInfo {
            title: "NAWA Web Operating System".into(),
            description: "Revolutionary Web Operating System — zero polling, real-time push, quantum-inspired computing".into(),
            version: "2.2.0".into(),
            license: Some(OpenApiLicense {
                name: "MIT OR Apache-2.0".into(),
                url: "https://github.com/amir-helal-ali/nawa-web-os".into(),
            }),
        },
        paths,
        components: Components {
            security_schemes: BTreeMap::from([
                ("bearerAuth".to_string(), SecurityScheme {
                    scheme_type: "http".into(),
                    scheme: "bearer".into(),
                    bearer_format: "JWT".into(),
                }),
            ]),
        },
    }
}

/// Convert NAWA path to OpenAPI path format.
fn convert_path(path: &str) -> String {
    // :param → {param}
    let mut result = path.to_string();
    while let Some(start) = result.find(':') {
        let end = result[start..].find('/').map(|e| start + e).unwrap_or(result.len());
        let param_name = &result[start+1..end];
        let placeholder = format!("{{{param_name}}}");
        result = format!("{}{}{}", &result[..start], placeholder, &result[end..]);
    }
    // ** → {path} (catch-all)
    result = result.replace("**", "{path}");
    result
}

/// Categorize an endpoint for documentation.
fn categorize_endpoint(path: &str) -> (String, String, String) {
    if path.starts_with("/api/quantum") {
        ("Quantum".into(), "Quantum operation".into(), "Quantum-inspired computing endpoint".into())
    } else if path.starts_with("/api/scheduler") {
        ("Scheduler".into(), "Task scheduling".into(), "Manage scheduled tasks".into())
    } else if path.starts_with("/api/notification") {
        ("Notifications".into(), "Notification management".into(), "Multi-channel notification system".into())
    } else if path.starts_with("/api/cache") {
        ("Cache".into(), "Cache management".into(), "Response cache statistics".into())
    } else if path.starts_with("/api/rate-limit") {
        ("Rate Limiting".into(), "Rate limiter stats".into(), "Sliding window rate limiter statistics".into())
    } else if path.starts_with("/api/audit") {
        ("Security".into(), "Audit log".into(), "Security audit log entries".into())
    } else if path.starts_with("/api/csrf") {
        ("Security".into(), "CSRF token".into(), "Generate CSRF protection token".into())
    } else if path.starts_with("/api/health") || path == "/health" {
        ("System".into(), "Health check".into(), "System health status".into())
    } else if path.starts_with("/api/stability") {
        ("System".into(), "Stability info".into(), "System stability features".into())
    } else if path.starts_with("/api/traces") {
        ("Monitoring".into(), "Request traces".into(), "Recent request traces with timing".into())
    } else if path.starts_with("/api/version") {
        ("System".into(), "API versioning".into(), "API version information".into())
    } else if path.starts_with("/api/middleware") {
        ("System".into(), "Middleware chain".into(), "Active middleware list".into())
    } else if path.starts_with("/auth") || path == "/register" || path == "/login" || path == "/logout" {
        ("Authentication".into(), "Auth operation".into(), "User authentication endpoint".into())
    } else if path.starts_with("/api/wasm-ssr") {
        ("WASM SSR".into(), "WASM server-side rendering".into(), "Render HTML via WASM module".into())
    } else if path.starts_with("/svelte") {
        ("SvelteKit".into(), "SvelteKit integration".into(), "Embedded SvelteKit application".into())
    } else if path.starts_with("/__photon__") || path.starts_with("/sitemap") || path.starts_with("/robots") || path.starts_with("/aion") {
        ("AION SEO".into(), "SEO endpoint".into(), "AION SEO Engine endpoint".into())
    } else if path == "/" {
        ("Dashboard".into(), "Main dashboard".into(), "NAWA web dashboard".into())
    } else if path == "/api" {
        ("System".into(), "API info".into(), "List all available endpoints".into())
    } else if path == "/system" {
        ("System".into(), "System info".into(), "System information and stats".into())
    } else if path == "/metrics" {
        ("Monitoring".into(), "Prometheus metrics".into(), "Prometheus-format metrics".into())
    } else {
        ("General".into(), "Endpoint".into(), format!("NAWA endpoint: {path}"))
    }
}

/// Build response codes for an endpoint.
fn build_responses(path: &str) -> BTreeMap<String, Response> {
    let mut responses = BTreeMap::new();
    responses.insert("200".into(), Response { description: "Successful response".into() });

    if needs_auth(path) {
        responses.insert("401".into(), Response { description: "Unauthorized — authentication required".into() });
        responses.insert("403".into(), Response { description: "Forbidden — admin access required".into() });
    }

    if path.contains(":") || path.contains("**") {
        responses.insert("404".into(), Response { description: "Resource not found".into() });
    }

    responses.insert("429".into(), Response { description: "Rate limit exceeded".into() });
    responses.insert("500".into(), Response { description: "Internal server error".into() });

    responses
}

/// Check if an endpoint requires authentication.
fn needs_auth(path: &str) -> bool {
    matches!(path,
        "/profile" | "/settings" | "/backup" | "/restore" |
        "/api/audit" | "/api/notifications/send"
    ) || path.starts_with("/admin/")
}

/// Generate Swagger UI HTML page.
pub fn swagger_ui_html(spec_url: &str) -> String {
    format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<title>NAWA API Documentation</title>
<link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/swagger-ui-dist@5/swagger-ui.css">
<style>
  body {{ margin: 0; }}
  .swagger-ui .topbar {{ background-color: #0a0a0f; }}
  .swagger-ui .topbar .download-url-wrapper {{ display: none; }}
</style>
</head>
<body>
<div id="swagger-ui"></div>
<script src="https://cdn.jsdelivr.net/npm/swagger-ui-dist@5/swagger-ui-bundle.js"></script>
<script>
  window.onload = function() {{
    SwaggerUIBundle({{
      url: "{spec_url}",
      dom_id: '#swagger-ui',
      deepLinking: true,
      presets: [SwaggerUIBundle.presets.apis],
      layout: "BaseLayout"
    }});
  }};
</script>
</body>
</html>"#, spec_url = spec_url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_spec_generates_valid_structure() {
        let endpoints = vec!["GET /", "GET /health", "POST /register", "GET /api/quantum"];
        let spec = build_spec(&endpoints);
        assert_eq!(spec.openapi, "3.0.3");
        assert!(spec.paths.contains_key("/"));
        assert!(spec.paths.contains_key("/health"));
        assert!(spec.paths.contains_key("/register"));
        assert!(spec.paths.contains_key("/api/quantum"));
    }

    #[test]
    fn convert_path_handles_params() {
        assert_eq!(convert_path("/users/:id"), "/users/{id}");
        assert_eq!(convert_path("/:key"), "/{key}");
        assert_eq!(convert_path("/scan/:prefix"), "/scan/{prefix}");
    }

    #[test]
    fn convert_path_handles_catchall() {
        assert_eq!(convert_path("/svelte/**"), "/svelte/{path}");
    }

    #[test]
    fn categorize_quantum_endpoints() {
        let (tag, _, _) = categorize_endpoint("/api/quantum");
        assert_eq!(tag, "Quantum");
    }

    #[test]
    fn categorize_auth_endpoints() {
        let (tag, _, _) = categorize_endpoint("/login");
        assert_eq!(tag, "Authentication");
    }

    #[test]
    fn needs_auth_detects_protected() {
        assert!(needs_auth("/profile"));
        assert!(needs_auth("/settings"));
        assert!(needs_auth("/admin/verify"));
        assert!(!needs_auth("/health"));
        assert!(!needs_auth("/api/quantum"));
    }

    #[test]
    fn build_responses_includes_401_for_auth() {
        let responses = build_responses("/profile");
        assert!(responses.contains_key("401"));
        assert!(responses.contains_key("403"));
    }

    #[test]
    fn build_responses_includes_404_for_params() {
        let responses = build_responses("/users/:id");
        assert!(responses.contains_key("404"));
    }

    #[test]
    fn swagger_ui_html_contains_spec_url() {
        let html = swagger_ui_html("/openapi.json");
        assert!(html.contains("/openapi.json"));
        assert!(html.contains("swagger-ui"));
    }

    #[test]
    fn spec_has_security_scheme() {
        let endpoints = vec!["GET /health"];
        let spec = build_spec(&endpoints);
        assert!(spec.components.security_schemes.contains_key("bearerAuth"));
    }
}
