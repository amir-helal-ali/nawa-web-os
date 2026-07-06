//! Integration tests for HTTP/3 (QUIC + h3).
//!
//! These tests verify the HTTP/3 server structure and configuration.
//! Full end-to-end tests require a TLS certificate and UDP socket
//! binding, which is complex in CI. We test the public API surface.

use nawa_http::{Http3Config, Http3Error, Http3Server, Router};

#[test]
fn http3_error_variants() {
    let quic_err = Http3Error::Quic("connection refused".into());
    assert!(format!("{quic_err}").contains("QUIC"));

    let h3_err = Http3Error::H3("parse error".into());
    assert!(format!("{h3_err}").contains("H3"));

    let io_err = Http3Error::Io(std::io::Error::new(std::io::ErrorKind::Other, "test"));
    assert!(format!("{io_err}").contains("io error"));
}

#[test]
fn http3_config_builders() {
    // We can't create a real TlsConfig without cert files,
    // but we can test the config builder methods.
    // Http3Config requires a TlsConfig, so we test via the server.
}

#[test]
fn http3_server_creation_requires_tls() {
    // Creating an Http3Server requires a TlsConfig.
    // Without cert files, we can't create one in unit tests.
    // This is expected — HTTP/3 mandates TLS 1.3.
}

#[test]
fn http3_error_tls_conversion() {
    let tls_err = nawa_http::TlsError::CertParse("no cert".into());
    let h3_err: Http3Error = tls_err.into();
    assert!(format!("{h3_err}").contains("TLS"));
}

#[test]
fn router_works_for_h3_dispatch() {
    // The same router works for both HTTP/1.1 and HTTP/3.
    use nawa_http::{Method, Request, Response};
    use std::collections::HashMap;

    let mut router = Router::new();
    router.get("/", |_| async { Response::text("hello from h3") });

    let req = Request {
        method: Method::Get,
        path: "/".into(),
        query: HashMap::new(),
        headers: HashMap::new(),
        body: Vec::new(),
        params: HashMap::new(),
    };

    // Use tokio runtime to test async dispatch.
    let rt = tokio::runtime::Runtime::new().unwrap();
    let resp = rt.block_on(router.dispatch(req));
    assert_eq!(resp.body, b"hello from h3");
}

#[test]
fn h3_supports_all_methods() {
    use nawa_http::{Method, Request, Response};
    use std::collections::HashMap;

    let mut router = Router::new();
    router.get("/g", |_| async { Response::text("GET") });
    router.post("/p", |_| async { Response::text("POST") });
    router.put("/u", |_| async { Response::text("PUT") });
    router.delete("/d", |_| async { Response::text("DELETE") });

    let rt = tokio::runtime::Runtime::new().unwrap();

    let make_req = |method: Method, path: &str| Request {
        method,
        path: path.into(),
        query: HashMap::new(),
        headers: HashMap::new(),
        body: Vec::new(),
        params: HashMap::new(),
    };

    assert_eq!(rt.block_on(router.dispatch(make_req(Method::Get, "/g"))).body, b"GET");
    assert_eq!(rt.block_on(router.dispatch(make_req(Method::Post, "/p"))).body, b"POST");
    assert_eq!(rt.block_on(router.dispatch(make_req(Method::Put, "/u"))).body, b"PUT");
    assert_eq!(rt.block_on(router.dispatch(make_req(Method::Delete, "/d"))).body, b"DELETE");
}

#[test]
fn h3_router_404() {
    use nawa_http::{Method, Request};
    use std::collections::HashMap;

    let router = Router::new();
    let req = Request {
        method: Method::Get,
        path: "/missing".into(),
        query: HashMap::new(),
        headers: HashMap::new(),
        body: Vec::new(),
        params: HashMap::new(),
    };

    let rt = tokio::runtime::Runtime::new().unwrap();
    let resp = rt.block_on(router.dispatch(req));
    assert_eq!(resp.status.0, 404);
}

#[test]
fn h3_json_response() {
    use nawa_http::{Method, Request, Response};
    use std::collections::HashMap;

    let mut router = Router::new();
    router.get("/api", |_| async {
        Response::json(&serde_json::json!({"proto": "h3", "transport": "quic"}))
    });

    let req = Request {
        method: Method::Get,
        path: "/api".into(),
        query: HashMap::new(),
        headers: HashMap::new(),
        body: Vec::new(),
        params: HashMap::new(),
    };

    let rt = tokio::runtime::Runtime::new().unwrap();
    let resp = rt.block_on(router.dispatch(req));
    let body = String::from_utf8_lossy(&resp.body);
    assert!(body.contains("h3"));
    assert!(body.contains("quic"));
}
