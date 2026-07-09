//! NAWA Performance Benchmarks
//!
//! Comprehensive benchmarks for all NAWA subsystems:
//! - nawa-db: PUT/GET/SCAN/DELETE operations
//! - nawa-http: routing dispatch + response writing
//! - nawa-engine: ZeroCopyHtml rendering
//! - nawa-aion: Knowledge Graph building + Photon response
//! - nawa-auth: password hashing + JWT verification
//! - nawa-svelte: manifest route matching
//!
//! Run with: cargo bench --bench nawa_benchmarks

use std::hint::black_box;
use std::time::Instant;

fn main() {
    println!("╔══════════════════════════════════════════════════════╗");
    println!("║  NAWA Performance Benchmarks                          ║");
    println!("╚══════════════════════════════════════════════════════╝");
    println!();

    bench_db();
    bench_engine();
    bench_aion();
    bench_auth();
    bench_svelte();
    bench_http_router();

    println!("╔══════════════════════════════════════════════════════╗");
    println!("║  All benchmarks complete                              ║");
    println!("╚══════════════════════════════════════════════════════╝");
}

fn bench_db() {
    println!("═══ NAWA-DB ═══");
    let db = nawa_db::DbEngine::open_in_memory();

    // PUT benchmark
    let n = 100_000;
    let start = Instant::now();
    for i in 0..n {
        let _ = db.put(format!("bench:{i}"), nawa_db::Value::from_i64(i as i64));
    }
    let elapsed = start.elapsed();
    println!("  PUT  {:>7} ops in {:>8.2?} → {:>8.0} ops/sec",
        n, elapsed, n as f64 / elapsed.as_secs_f64());

    // GET benchmark (random hits)
    let start = Instant::now();
    let mut hits = 0;
    for i in 0..n {
        if db.get(format!("bench:{i}")).is_some() { hits += 1; }
    }
    let elapsed = start.elapsed();
    println!("  GET  {:>7} ops in {:>8.2?} → {:>8.0} ops/sec ({hits} hits)",
        n, elapsed, n as f64 / elapsed.as_secs_f64());

    // SCAN benchmark
    let start = Instant::now();
    let results = db.scan_prefix("bench:", 1_000_000);
    let elapsed = start.elapsed();
    println!("  SCAN {:>7} hits in {:>8.2?} → {:>8.0} ops/sec",
        results.len(), elapsed, results.len() as f64 / elapsed.as_secs_f64());

    // DELETE benchmark
    let start = Instant::now();
    for i in 0..n {
        let _ = db.delete(format!("bench:{i}"));
    }
    let elapsed = start.elapsed();
    println!("  DEL  {:>7} ops in {:>8.2?} → {:>8.0} ops/sec",
        n, elapsed, n as f64 / elapsed.as_secs_f64());
    println!();
}

fn bench_engine() {
    println!("═══ NAWA-Engine (ZeroCopyHtml) ═══");
    use nawa_engine::UnifiedEngine;

    let db = nawa_db::DbEngine::open_in_memory();
    let _ = db.put("title", nawa_db::Value::from_str("NAWA Benchmark"));
    let _ = db.put("body", nawa_db::Value::from_str("This is a benchmark test for the NAWA engine."));

    let ctx = nawa_engine::EngineContext::new(std::sync::Arc::new(db));

    let n = 10_000;
    let start = Instant::now();
    for _ in 0..n {
        let result = UnifiedEngine::render_db_page(black_box(&ctx), black_box("NAWA Bench"));
        black_box(result);
    }
    let elapsed = start.elapsed();
    println!("  Render {:>6} pages in {:>8.2?} → {:>8.0} pages/sec",
        n, elapsed, n as f64 / elapsed.as_secs_f64());
    println!();
}

fn bench_aion() {
    println!("═══ AION Engine ═══");
    let db = nawa_db::DbEngine::open_in_memory();

    // Seed DB with 1000 entities.
    for i in 0..1000 {
        let _ = db.put(
            format!("user:{i}"),
            nawa_db::Value::from_json_str(&format!(
                r#"{{"username":"user{i}","email":"u{i}@nawa.test","password_hash":"x","name":"User {i}"}}"#
            )).unwrap()
        );
    }
    for i in 0..500 {
        let _ = db.put(
            format!("post:{i}"),
            nawa_db::Value::from_json_str(&format!(
                r#"{{"title":"Post {i}","description":"Description {i}","author":"{i}","date_published":"2026-01-01"}}"#
            )).unwrap()
        );
    }

    // Knowledge Graph building
    let n = 10;
    let start = Instant::now();
    for _ in 0..n {
        let graph = nawa_aion::build_knowledge_graph(black_box(&db));
        black_box(graph);
    }
    let elapsed = start.elapsed();
    println!("  Knowledge Graph (1500 entities): 10 builds in {:>8.2?} → {:.2} ms/build",
        elapsed, elapsed.as_secs_f64() * 1000.0 / n as f64);

    // Photon response
    let graph = nawa_aion::build_knowledge_graph(&db);
    let start = Instant::now();
    for _ in 0..n {
        let photon = nawa_aion::build_photon_response(black_box(&graph), black_box("https://nawa.dev"));
        black_box(photon);
    }
    let elapsed = start.elapsed();
    println!("  Photon Protocol response:        10 builds in {:>8.2?} → {:.2} ms/build",
        elapsed, elapsed.as_secs_f64() * 1000.0 / n as f64);

    // Negotiation
    let ua = "Mozilla/5.0 (compatible; Googlebot/2.1)";
    let accept = "text/html";
    let n = 100_000;
    let start = Instant::now();
    for _ in 0..n {
        let fmt = nawa_aion::negotiate(black_box(ua), black_box(accept));
        black_box(fmt);
    }
    let elapsed = start.elapsed();
    println!("  Negotiation:     {:>6} calls in {:>8.2?} → {:>8.0} calls/sec",
        n, elapsed, n as f64 / elapsed.as_secs_f64());
    println!();
}

fn bench_auth() {
    println!("═══ NAWA-Auth ═══");
    use nawa_auth::{password, AuthConfig, AuthStore};
    let db = std::sync::Arc::new(nawa_db::DbEngine::open_in_memory());
    let auth = AuthStore::new(db, AuthConfig::with_secret("bench_secret"));

    // Password hashing
    let n = 100;
    let start = Instant::now();
    for i in 0..n {
        let hash = password::hash_password(black_box(&format!("password{i}")));
        black_box(hash);
    }
    let elapsed = start.elapsed();
    println!("  Password hashing: {:>4} hashes in {:>8.2?} → {:.2} ms/hash",
        n, elapsed, elapsed.as_secs_f64() * 1000.0 / n as f64);

    // JWT verification
    let _ = auth.register("benchuser", "bench@nawa.test", "pass123").unwrap();
    let token = auth.login("bench@nawa.test", "pass123").unwrap().token;

    let n = 10_000;
    let start = Instant::now();
    for _ in 0..n {
        let claims = auth.verify_token(black_box(&token));
        let _ = black_box(claims);
    }
    let elapsed = start.elapsed();
    println!("  JWT verification: {:>5} calls in {:>8.2?} → {:>8.0} calls/sec",
        n, elapsed, n as f64 / elapsed.as_secs_f64());
    println!();
}

fn bench_svelte() {
    println!("═══ NAWA-Svelte ═══");
    let manifest_json = r#"{
        "version": 1, "app_name": "Bench", "built_at": "t", "sveltekit_version": "1",
        "routes": [
            {"pattern":"/","methods":["GET"],"prerendered_html":"index.html","hydration_js":null,"requires_auth":false,"admin_only":false,"meta":{},"is_endpoint":false,"ssr_wasm":null,"layout":null},
            {"pattern":"/users/[id]","methods":["GET"],"prerendered_html":null,"hydration_js":"users.js","requires_auth":true,"admin_only":false,"meta":{},"is_endpoint":false,"ssr_wasm":null,"layout":null},
            {"pattern":"/blog/[...slug]","methods":["GET"],"prerendered_html":null,"hydration_js":null,"requires_auth":false,"admin_only":false,"meta":{},"is_endpoint":false,"ssr_wasm":null,"layout":null},
            {"pattern":"/api/data","methods":["GET","POST"],"prerendered_html":null,"hydration_js":null,"requires_auth":false,"admin_only":false,"meta":{},"is_endpoint":true,"ssr_wasm":null,"layout":null}
        ],
        "global_css": "app.css", "main_js": "app.js", "favicon": null,
        "spa_fallback": "spa.html", "default_meta": {}
    }"#;
    let manifest: nawa_svelte::NawaManifest = serde_json::from_str(manifest_json).unwrap();

    let n = 100_000;
    let test_paths = ["/", "/users/42", "/blog/2026/hello", "/api/data", "/nonexistent"];
    let start = Instant::now();
    for i in 0..n {
        let path = black_box(test_paths[i % test_paths.len()]);
        let matched = manifest.match_route(path);
        black_box(matched);
    }
    let elapsed = start.elapsed();
    println!("  Route matching: {:>6} calls in {:>8.2?} → {:>8.0} calls/sec",
        n, elapsed, n as f64 / elapsed.as_secs_f64());
    println!();
}

fn bench_http_router() {
    println!("═══ NAWA-HTTP Router ═══");
    use nawa_http::Router;

    let mut router = Router::new();
    router.get("/", |_| async { nawa_http::Response::text("home") });
    router.get("/users/:id", |_| async { nawa_http::Response::text("user") });
    router.get("/posts/:slug", |_| async { nawa_http::Response::text("post") });
    router.get("/health", |_| async { nawa_http::Response::text("ok") });
    router.get("/static/**", |_| async { nawa_http::Response::text("asset") });

    let routes = ["/", "/users/42", "/posts/hello-world", "/health", "/static/css/app.css"];
    let n = 50_000;

    // Build requests
    let requests: Vec<_> = (0..n)
        .map(|i| nawa_http::Request {
            method: nawa_http::Method::Get,
            path: routes[i % routes.len()].to_string(),
            query: Default::default(),
            headers: Default::default(),
            body: Vec::new(),
            params: Default::default(),
        })
        .collect();

    let start = Instant::now();
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        for req in requests {
            let resp = router.dispatch(black_box(req)).await;
            black_box(resp);
        }
    });
    let elapsed = start.elapsed();
    println!("  Dispatch:       {:>6} calls in {:>8.2?} → {:>8.0} calls/sec",
        n, elapsed, n as f64 / elapsed.as_secs_f64());
    println!();
}
