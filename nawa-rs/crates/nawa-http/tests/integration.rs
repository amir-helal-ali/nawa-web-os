//! Integration tests for nawa-http router.

use nawa_http::{Method, Request, Response, Router, StatusCode};
use std::collections::HashMap;

fn make_request(method: Method, path: &str) -> Request {
    Request {
        method,
        path: path.to_string(),
        query: HashMap::new(),
        headers: HashMap::new(),
        body: Vec::new(),
        params: HashMap::new(),
    }
}

#[tokio::test]
async fn root_route() {
    let mut router = Router::new();
    router.get("/", |_| async { Response::text("NAWA v0.1.0") });

    let resp = router.dispatch(make_request(Method::Get, "/")).await;
    assert_eq!(resp.status.0, 200);
    assert_eq!(resp.body, b"NAWA v0.1.0");
}

#[tokio::test]
async fn param_extraction() {
    let mut router = Router::new();
    router.get("/users/:id", |req| async move {
        let id = req.param("id").unwrap_or("?");
        Response::text(format!("user-{id}"))
    });

    let resp = router
        .dispatch(make_request(Method::Get, "/users/1001"))
        .await;
    assert_eq!(resp.body, b"user-1001");
}

#[tokio::test]
async fn nested_params() {
    let mut router = Router::new();
    router.get("/users/:user_id/posts/:post_id", |req| async move {
        let uid = req.param("user_id").unwrap_or("?");
        let pid = req.param("post_id").unwrap_or("?");
        Response::text(format!("{uid}/{pid}"))
    });

    let resp = router
        .dispatch(make_request(Method::Get, "/users/42/posts/99"))
        .await;
    assert_eq!(resp.body, b"42/99");
}

#[tokio::test]
async fn method_routing() {
    let mut router = Router::new();
    router.get("/items", |_| async { Response::text("list") });
    router.post("/items", |_| async { Response::text("created") });
    router.put("/items/:id", |_| async { Response::text("updated") });
    router.delete("/items/:id", |_| async { Response::text("deleted") });

    assert_eq!(
        router.dispatch(make_request(Method::Get, "/items")).await.body,
        b"list"
    );
    assert_eq!(
        router.dispatch(make_request(Method::Post, "/items")).await.body,
        b"created"
    );
    assert_eq!(
        router
            .dispatch(make_request(Method::Put, "/items/1"))
            .await
            .body,
        b"updated"
    );
    assert_eq!(
        router
            .dispatch(make_request(Method::Delete, "/items/1"))
            .await
            .body,
        b"deleted"
    );
}

#[tokio::test]
async fn not_found() {
    let router = Router::new();
    let resp = router
        .dispatch(make_request(Method::Get, "/missing"))
        .await;
    assert_eq!(resp.status.0, 404);
}

#[tokio::test]
async fn json_response() {
    let mut router = Router::new();
    router.get("/api", |_| async {
        Response::json(&serde_json::json!({
            "name": "NAWA",
            "version": "0.1.0"
        }))
    });

    let resp = router.dispatch(make_request(Method::Get, "/api")).await;
    assert_eq!(resp.status.0, 200);
    let body_str = String::from_utf8_lossy(&resp.body);
    assert!(body_str.contains("NAWA"));
    assert!(body_str.contains("0.1.0"));
}

#[tokio::test]
async fn multiple_routes_no_conflict() {
    let mut router = Router::new();
    router.get("/users", |_| async { Response::text("list") });
    router.get("/users/:id", |_| async { Response::text("one") });
    router.get("/users/:id/posts", |_| async { Response::text("posts") });

    assert_eq!(
        router.dispatch(make_request(Method::Get, "/users")).await.body,
        b"list"
    );
    assert_eq!(
        router
            .dispatch(make_request(Method::Get, "/users/1"))
            .await
            .body,
        b"one"
    );
    assert_eq!(
        router
            .dispatch(make_request(Method::Get, "/users/1/posts"))
            .await
            .body,
        b"posts"
    );
}

#[tokio::test]
async fn response_with_custom_status() {
    let mut router = Router::new();
    router.get("/teapot", |_| async {
        Response::new(StatusCode(418)).status(StatusCode(418))
    });

    let resp = router.dispatch(make_request(Method::Get, "/teapot")).await;
    assert_eq!(resp.status.0, 418);
}
