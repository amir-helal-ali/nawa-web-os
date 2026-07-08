# nawa-http

HTTP/1.1 + HTTP/3 server with type-safe router, TLS, and ACME.

## Router

```rust
use nawa_http::{Router, Response, Method};

let mut router = Router::new();

// Literal routes
router.get("/", |_| async { Response::text("hello") });

// Parameterized routes
router.get("/users/:id", |req| async move {
    let id = req.param("id").unwrap_or("?");
    Response::text(format!("user-{id}"))
});

// Nested params
router.get("/users/:uid/posts/:pid", |req| async move {
    Response::text(format!("{}/{}", req.param("uid")?, req.param("pid")?))
});

// All HTTP methods
router.post("/items", |_| async { Response::text("created") });
router.put("/items/:id", |_| async { Response::text("updated") });
router.delete("/items/:id", |_| async { Response::text("deleted") });
```

## HTTP/1.1 Server

```rust
use nawa_http::HttpServer;
use std::net::SocketAddr;

let addr: SocketAddr = "0.0.0.0:8080".parse()?;
let server = HttpServer::new(router, addr);
server.serve().await?;
```

## HTTP/3 Server (QUIC)

```rust
use nawa_http::{Http3Server, Http3Config, TlsConfig};

let tls = TlsConfig::from_pem_files("cert.pem", "key.pem")?;
let config = Http3Config::new("[::]:8443".parse()?, tls)
    .with_max_streams(200);

let server = Http3Server::new(config, router);
server.serve().await?; // serves HTTP/3 over QUIC
```

## TLS / HTTPS

```rust
use nawa_http::TlsConfig;

let tls = TlsConfig::from_pem_files("cert.pem", "key.pem")?;
```

## Let's Encrypt (ACME)

```rust
use nawa_http::{AcmeClient, AcmeConfig};

let config = AcmeConfig::new(
    vec!["example.com".into()],
    "admin@example.com".into(),
    "/var/lib/nawa/certs".into(),
);
let client = AcmeClient::new(config);

// Register HTTP-01 challenge
client.register_challenge("token123", "key-auth-xxx");

// Serve challenge response
let response = client.get_challenge_response("token123");

// Provision cert (loads existing or triggers flow)
let tls = client.provision().await?;
```

## Modules

| Module | Description |
|--------|-------------|
| router | Type-safe routing with `:params` and `*wildcards` |
| server | HTTP/1.1 over TCP, keep-alive, response timing |
| h3 | HTTP/3 over QUIC (quinn + h3) |
| tls | rustls-based TLS config |
| acme | Let's Encrypt auto-TLS provisioning |

## Tests
- 15 unit tests
- 8 integration tests
