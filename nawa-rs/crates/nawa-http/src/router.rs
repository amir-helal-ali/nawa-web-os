//! Type-safe HTTP router.

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// HTTP method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Method {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Head,
    Options,
}

impl Method {
    pub fn as_str(&self) -> &'static str {
        match self {
            Method::Get => "GET",
            Method::Post => "POST",
            Method::Put => "PUT",
            Method::Delete => "DELETE",
            Method::Patch => "PATCH",
            Method::Head => "HEAD",
            Method::Options => "OPTIONS",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "GET" => Some(Method::Get),
            "POST" => Some(Method::Post),
            "PUT" => Some(Method::Put),
            "DELETE" => Some(Method::Delete),
            "PATCH" => Some(Method::Patch),
            "HEAD" => Some(Method::Head),
            "OPTIONS" => Some(Method::Options),
            _ => None,
        }
    }
}

/// HTTP status code.
#[derive(Debug, Clone, Copy)]
pub struct StatusCode(pub u16);

impl StatusCode {
    pub const OK: Self = StatusCode(200);
    pub const NOT_FOUND: Self = StatusCode(404);
    pub const BAD_REQUEST: Self = StatusCode(400);
    pub const INTERNAL_SERVER_ERROR: Self = StatusCode(500);
    pub const UNAUTHORIZED: Self = StatusCode(401);
    pub const FORBIDDEN: Self = StatusCode(403);
    pub const CREATED: Self = StatusCode(201);
    pub const NO_CONTENT: Self = StatusCode(204);
}

impl StatusCode {
    pub fn reason_phrase(&self) -> &'static str {
        match self.0 {
            200 => "OK",
            201 => "Created",
            204 => "No Content",
            400 => "Bad Request",
            401 => "Unauthorized",
            403 => "Forbidden",
            404 => "Not Found",
            500 => "Internal Server Error",
            _ => "Unknown",
        }
    }
}

/// An incoming HTTP request.
#[derive(Debug, Clone)]
pub struct Request {
    pub method: Method,
    pub path: String,
    pub query: HashMap<String, String>,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub params: HashMap<String, String>,
}

impl Request {
    /// Get a path parameter (e.g., `/users/:id` → `params["id"]`).
    pub fn param(&self, name: &str) -> Option<&str> {
        self.params.get(name).map(|s| s.as_str())
    }

    /// Get a query parameter.
    pub fn query(&self, name: &str) -> Option<&str> {
        self.query.get(name).map(|s| s.as_str())
    }

    /// Get a header.
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(&name.to_lowercase()).map(|s| s.as_str())
    }

    /// Body as UTF-8 string.
    pub fn body_str(&self) -> &str {
        std::str::from_utf8(&self.body).unwrap_or("")
    }

    /// Body parsed as JSON.
    pub fn json<T: serde::de::DeserializeOwned>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_slice(&self.body)
    }
}

/// An HTTP response.
#[derive(Debug, Clone)]
pub struct Response {
    pub status: StatusCode,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl Response {
    pub fn new(status: StatusCode) -> Self {
        Self {
            status,
            headers: HashMap::new(),
            body: Vec::new(),
        }
    }

    pub fn ok(body: impl Into<Vec<u8>>) -> Self {
        let mut r = Self::new(StatusCode::OK);
        r.body = body.into();
        r
    }

    pub fn text(s: impl Into<String>) -> Self {
        let mut r = Self::new(StatusCode::OK);
        r.header("Content-Type", "text/plain; charset=utf-8");
        r.body = s.into().into_bytes();
        r
    }

    pub fn json<T: serde::Serialize>(value: &T) -> Self {
        let mut r = Self::new(StatusCode::OK);
        r.header("Content-Type", "application/json");
        r.body = serde_json::to_vec(value).unwrap_or_default();
        r
    }

    pub fn not_found(msg: impl Into<String>) -> Self {
        let mut r = Self::new(StatusCode::NOT_FOUND);
        r.body = msg.into().into_bytes();
        r
    }

    pub fn header(&mut self, name: &str, value: &str) -> &mut Self {
        self.headers.insert(name.to_string(), value.to_string());
        self
    }

    pub fn status(mut self, status: StatusCode) -> Self {
        self.status = status;
        self
    }
}

/// A handler is an async function that takes a Request and returns a Response.
pub type Handler = Arc<
    dyn Fn(Request) -> Pin<Box<dyn Future<Output = Response> + Send>> + Send + Sync,
>;

/// A route — method + path pattern + handler.
struct Route {
    method: Method,
    pattern: PathPattern,
    handler: Handler,
}

/// A compiled path pattern like `/users/:id/posts/:post_id`.
#[derive(Debug, Clone)]
struct PathPattern {
    segments: Vec<PatternSegment>,
}

#[derive(Debug, Clone)]
enum PatternSegment {
    Literal(String),
    Param(String),
    Wildcard,        // matches one segment (e.g., `/static/*`)
    CatchAll(String), // matches rest of path (e.g., `/svelte/**` → captures as named param)
}

impl PathPattern {
    fn parse(pattern: &str) -> Self {
        let segments = pattern
            .split('/')
            .filter(|s| !s.is_empty())
            .map(|s| {
                if let Some(rest) = s.strip_prefix("**") {
                    // Catch-all: ** or **name
                    let name = if rest.is_empty() { "_rest".to_string() } else { rest.to_string() };
                    PatternSegment::CatchAll(name)
                } else if let Some(param) = s.strip_prefix(':') {
                    PatternSegment::Param(param.to_string())
                } else if s == "*" {
                    PatternSegment::Wildcard
                } else {
                    PatternSegment::Literal(s.to_string())
                }
            })
            .collect();
        Self { segments }
    }

    fn matches(&self, path: &str) -> Option<HashMap<String, String>> {
        let path_segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        let mut params = HashMap::new();

        // Check if the last segment is a catch-all.
        let has_catchall = matches!(
            self.segments.last(),
            Some(PatternSegment::CatchAll(_))
        );

        if !has_catchall {
            // Strict length match for non-catchall patterns.
            if path_segments.len() != self.segments.len() {
                return None;
            }
        } else {
            // Catch-all: path must have AT LEAST as many segments as pattern (minus the catchall).
            // The catch-all itself can match zero segments (e.g., /svelte/** matches /svelte).
            if path_segments.len() < self.segments.len() - 1 {
                return None;
            }
        }

        for (i, seg) in self.segments.iter().enumerate() {
            match seg {
                PatternSegment::Literal(lit) => {
                    if path_segments.get(i) != Some(&lit.as_str()) {
                        return None;
                    }
                }
                PatternSegment::Param(name) => {
                    if let Some(ps) = path_segments.get(i) {
                        params.insert(name.clone(), ps.to_string());
                    } else {
                        return None;
                    }
                }
                PatternSegment::Wildcard => {
                    path_segments.get(i)?;
                }
                PatternSegment::CatchAll(name) => {
                    // Capture all remaining segments as a single slash-joined string.
                    let rest: Vec<&str> = path_segments[i..].to_vec();
                    if rest.is_empty() {
                        params.insert(name.clone(), String::new());
                    } else {
                        params.insert(name.clone(), rest.join("/"));
                    }
                    break; // catch-all consumes the rest
                }
            }
        }

        Some(params)
    }
}

/// The router — holds all registered routes.
pub struct Router {
    routes: Vec<Route>,
}

impl Router {
    pub fn new() -> Self {
        Self { routes: Vec::new() }
    }

    /// Register a GET handler.
    pub fn get<F, Fut>(&mut self, pattern: &str, handler: F) -> &mut Self
    where
        F: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        self.add_route(Method::Get, pattern, handler)
    }

    /// Register a POST handler.
    pub fn post<F, Fut>(&mut self, pattern: &str, handler: F) -> &mut Self
    where
        F: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        self.add_route(Method::Post, pattern, handler)
    }

    /// Register a PUT handler.
    pub fn put<F, Fut>(&mut self, pattern: &str, handler: F) -> &mut Self
    where
        F: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        self.add_route(Method::Put, pattern, handler)
    }

    /// Register a DELETE handler.
    pub fn delete<F, Fut>(&mut self, pattern: &str, handler: F) -> &mut Self
    where
        F: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        self.add_route(Method::Delete, pattern, handler)
    }

    fn add_route<F, Fut>(&mut self, method: Method, pattern: &str, handler: F) -> &mut Self
    where
        F: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        let handler: Handler = Arc::new(move |req| Box::pin(handler(req)));
        self.routes.push(Route {
            method,
            pattern: PathPattern::parse(pattern),
            handler,
        });
        self
    }

    /// Try to dispatch a request. Returns 404 if no route matches.
    pub async fn dispatch(&self, mut req: Request) -> Response {
        // Three-pass matching for correct priority:
        // Pass 0: pure-literal routes (e.g., /health, /svelte)
        // Pass 1: routes with params but NO catch-all (e.g., /:key, /users/:id)
        // Pass 2: routes with catch-all (e.g., /svelte/**)
        // This ensures /health beats /:key beats /svelte/** when they all could match.
        for pass in 0..3 {
            for route in &self.routes {
                if route.method != req.method {
                    continue;
                }
                let is_pure_literal = route.pattern.segments.iter().all(|s| {
                    matches!(s, PatternSegment::Literal(_))
                });
                let has_catchall = route.pattern.segments.iter().any(|s| {
                    matches!(s, PatternSegment::CatchAll(_))
                });

                // Pass 0: pure literals only.
                if pass == 0 && !is_pure_literal { continue; }
                // Pass 1: param/wildcard routes WITHOUT catch-all.
                if pass == 1 && (is_pure_literal || has_catchall) { continue; }
                // Pass 2: catch-all routes.
                if pass == 2 && !has_catchall { continue; }

                if let Some(params) = route.pattern.matches(&req.path) {
                    req.params = params;
                    return (route.handler)(req).await;
                }
            }
        }
        Response::not_found(format!("no route for {} {}", req.method.as_str(), req.path))
    }

    /// Number of registered routes.
    pub fn len(&self) -> usize {
        self.routes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.routes.is_empty()
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn dispatch_get_literal() {
        let mut router = Router::new();
        router.get("/", |_| async { Response::text("hello") });
        let req = Request {
            method: Method::Get,
            path: "/".into(),
            query: HashMap::new(),
            headers: HashMap::new(),
            body: Vec::new(),
            params: HashMap::new(),
        };
        let resp = router.dispatch(req).await;
        assert_eq!(resp.status.0, 200);
        assert_eq!(resp.body, b"hello");
    }

    #[tokio::test]
    async fn dispatch_get_with_params() {
        let mut router = Router::new();
        router.get("/users/:id", |req| async move {
            let id = req.param("id").unwrap_or("?");
            Response::text(format!("user {}", id))
        });
        let req = Request {
            method: Method::Get,
            path: "/users/1001".into(),
            query: HashMap::new(),
            headers: HashMap::new(),
            body: Vec::new(),
            params: HashMap::new(),
        };
        let resp = router.dispatch(req).await;
        assert_eq!(resp.body, b"user 1001");
    }

    #[tokio::test]
    async fn dispatch_404() {
        let router = Router::new();
        let req = Request {
            method: Method::Get,
            path: "/missing".into(),
            query: HashMap::new(),
            headers: HashMap::new(),
            body: Vec::new(),
            params: HashMap::new(),
        };
        let resp = router.dispatch(req).await;
        assert_eq!(resp.status.0, 404);
    }

    #[tokio::test]
    async fn method_mismatch_404() {
        let mut router = Router::new();
        router.get("/users", |_| async { Response::text("users") });
        let req = Request {
            method: Method::Post,
            path: "/users".into(),
            query: HashMap::new(),
            headers: HashMap::new(),
            body: Vec::new(),
            params: HashMap::new(),
        };
        let resp = router.dispatch(req).await;
        assert_eq!(resp.status.0, 404);
    }
}
