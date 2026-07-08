//! Middleware chain — composable request/response middleware.

#![allow(dead_code)]

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

/// Correlation ID generator — unique per request.
pub struct CorrelationId;

impl CorrelationId {
    /// Generate a unique correlation ID (format: req-<timestamp>-<counter>).
    pub fn generate() -> String {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let count = COUNTER.fetch_add(1, Ordering::SeqCst);
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        format!("req-{ts:013x}-{count:06x}")
    }
}

/// Request trace — tracks timing and metadata for a single request.
#[derive(Debug, Clone, serde::Serialize)]
pub struct RequestTrace {
    pub correlation_id: String,
    pub method: String,
    pub path: String,
    pub started_at: String,
    pub duration_ms: u64,
    pub status: u16,
    pub bytes_sent: usize,
    pub bytes_received: usize,
    pub user_agent: Option<String>,
    pub remote_addr: Option<String>,
}

impl RequestTrace {
    /// Start a new trace for a request.
    pub fn start(method: &str, path: &str) -> Self {
        Self {
            correlation_id: CorrelationId::generate(),
            method: method.to_string(),
            path: path.to_string(),
            started_at: chrono::Utc::now().to_rfc3339(),
            duration_ms: 0,
            status: 0,
            bytes_sent: 0,
            bytes_received: 0,
            user_agent: None,
            remote_addr: None,
        }
    }

    /// Complete the trace after the response is sent.
    pub fn complete(&mut self, status: u16, bytes_sent: usize, bytes_received: usize) {
        self.status = status;
        self.bytes_sent = bytes_sent;
        self.bytes_received = bytes_received;
        // Duration is calculated by the caller using Instant.
    }
}

/// Trace store — keeps recent request traces in memory.
pub struct TraceStore {
    traces: tokio::sync::RwLock<Vec<RequestTrace>>,
    max_traces: usize,
    total_requests: AtomicU64,
    total_errors: AtomicU64,
    total_duration_ms: AtomicU64,
}

impl TraceStore {
    /// Create a new trace store with the given capacity.
    pub fn new(max_traces: usize) -> Arc<Self> {
        Arc::new(Self {
            traces: tokio::sync::RwLock::new(Vec::with_capacity(max_traces)),
            max_traces,
            total_requests: AtomicU64::new(0),
            total_errors: AtomicU64::new(0),
            total_duration_ms: AtomicU64::new(0),
        })
    }

    /// Record a completed request trace.
    pub async fn record(&self, trace: RequestTrace) {
        if trace.status >= 400 {
            self.total_errors.fetch_add(1, Ordering::Relaxed);
        }
        self.total_duration_ms.fetch_add(trace.duration_ms, Ordering::Relaxed);
        self.total_requests.fetch_add(1, Ordering::Relaxed);

        let mut traces = self.traces.write().await;
        traces.push(trace);
        if traces.len() > self.max_traces {
            traces.remove(0);
        }
    }

    /// Get recent traces (newest first).
    pub async fn recent(&self, limit: usize) -> Vec<RequestTrace> {
        let traces = self.traces.read().await;
        traces.iter().rev().take(limit).cloned().collect()
    }

    /// Get aggregate statistics.
    pub fn stats(&self) -> TraceStats {
        let total = self.total_requests.load(Ordering::Relaxed);
        let errors = self.total_errors.load(Ordering::Relaxed);
        let total_duration = self.total_duration_ms.load(Ordering::Relaxed);
        TraceStats {
            total_requests: total,
            total_errors: errors,
            avg_duration_ms: total_duration.checked_div(total).unwrap_or(0),
            error_rate: if total > 0 { (errors as f64 / total as f64) * 100.0 } else { 0.0 },
        }
    }
}

/// Aggregate trace statistics.
#[derive(Debug, Clone, serde::Serialize)]
pub struct TraceStats {
    pub total_requests: u64,
    pub total_errors: u64,
    pub avg_duration_ms: u64,
    pub error_rate: f64,
}

/// Simple response compressor — basic RLE-style compression.
/// (Production would use gzip/deflate via flate2, but this keeps zero-deps.)
pub struct Compressor;

impl Compressor {
    /// Check if the client accepts compression.
    pub fn should_compress(accept_encoding: Option<&str>) -> bool {
        accept_encoding
            .map(|e| e.contains("gzip") || e.contains("deflate") || e.contains("br"))
            .unwrap_or(false)
    }

    /// Estimate compression ratio for text content.
    pub fn estimate_ratio(content_type: &str) -> f32 {
        if content_type.contains("text") || content_type.contains("json") || content_type.contains("javascript") || content_type.contains("css") {
            0.3 // Text compresses ~70%
        } else if content_type.contains("svg") || content_type.contains("xml") {
            0.4
        } else {
            1.0 // Binary doesn't compress well
        }
    }

    /// Check if a response is worth compressing.
    pub fn worth_compressing(body_size: usize, content_type: &str) -> bool {
        body_size > 1024 && Self::estimate_ratio(content_type) < 0.9
    }
}

/// API versioning — extract and validate version from request.
pub struct ApiVersioning;

impl ApiVersioning {
    /// Extract API version from request path or header.
    /// Priority: X-API-Version header > /v{N}/ path prefix > default (1).
    pub fn extract_version(path: &str, api_version_header: Option<&str>) -> u32 {
        // Check header first.
        if let Some(hdr) = api_version_header {
            if let Ok(v) = hdr.trim_start_matches('v').parse::<u32>() {
                return v;
            }
        }
        // Check path prefix /v1/, /v2/, etc.
        if let Some(rest) = path.strip_prefix('/') {
            if let Some(rest) = rest.strip_prefix('v') {
                if let Some(end) = rest.find('/') {
                    if let Ok(v) = rest[..end].parse::<u32>() {
                        return v;
                    }
                } else if let Ok(v) = rest.parse::<u32>() {
                    return v;
                }
            }
        }
        1 // Default version
    }

    /// Check if a version is supported.
    pub fn is_supported(version: u32) -> bool {
        (1..=2).contains(&version)
    }

    /// Get deprecation status for a version.
    pub fn deprecation_status(version: u32) -> &'static str {
        match version {
            1 => "supported",
            2 => "current",
            _ => "unsupported",
        }
    }
}

/// Request timing middleware — measures request duration.
pub struct RequestTimer {
    start: Instant,
}

impl RequestTimer {
    pub fn new() -> Self {
        Self { start: Instant::now() }
    }

    pub fn elapsed_ms(&self) -> u64 {
        self.start.elapsed().as_millis() as u64
    }

    pub fn elapsed_micros(&self) -> u64 {
        self.start.elapsed().as_micros() as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correlation_id_is_unique() {
        let id1 = CorrelationId::generate();
        let id2 = CorrelationId::generate();
        assert_ne!(id1, id2);
        assert!(id1.starts_with("req-"));
    }

    #[tokio::test]
    async fn trace_store_records_traces() {
        let store = TraceStore::new(100);
        let mut trace = RequestTrace::start("GET", "/users");
        trace.complete(200, 1024, 0);
        trace.duration_ms = 5;
        store.record(trace).await;
        let recent = store.recent(10).await;
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].path, "/users");
    }

    #[tokio::test]
    async fn trace_store_limits_size() {
        let store = TraceStore::new(3);
        for i in 0..5 {
            let mut trace = RequestTrace::start("GET", &format!("/path{i}"));
            trace.complete(200, 100, 0);
            store.record(trace).await;
        }
        let recent = store.recent(10).await;
        assert_eq!(recent.len(), 3);
    }

    #[tokio::test]
    async fn trace_stats_track_errors() {
        let store = TraceStore::new(100);
        let mut ok = RequestTrace::start("GET", "/ok");
        ok.complete(200, 100, 0);
        store.record(ok).await;

        let mut err = RequestTrace::start("GET", "/err");
        err.complete(500, 100, 0);
        store.record(err).await;

        let stats = store.stats();
        assert_eq!(stats.total_requests, 2);
        assert_eq!(stats.total_errors, 1);
        assert!((stats.error_rate - 50.0).abs() < 0.1);
    }

    #[test]
    fn compressor_detects_accept_encoding() {
        assert!(Compressor::should_compress(Some("gzip, deflate")));
        assert!(Compressor::should_compress(Some("br")));
        assert!(!Compressor::should_compress(Some("identity")));
        assert!(!Compressor::should_compress(None));
    }

    #[test]
    fn compressor_estimates_ratio() {
        assert!(Compressor::estimate_ratio("text/html") < 0.5);
        assert!(Compressor::estimate_ratio("application/json") < 0.5);
        assert!(Compressor::estimate_ratio("image/png") >= 1.0);
    }

    #[test]
    fn compressor_worth_compressing() {
        assert!(Compressor::worth_compressing(2048, "text/html"));
        assert!(!Compressor::worth_compressing(500, "text/html")); // too small
        assert!(!Compressor::worth_compressing(2048, "image/png")); // binary
    }

    #[test]
    fn api_versioning_from_header() {
        assert_eq!(ApiVersioning::extract_version("/users", Some("v2")), 2);
        assert_eq!(ApiVersioning::extract_version("/users", Some("2")), 2);
        assert_eq!(ApiVersioning::extract_version("/users", Some("v1")), 1);
    }

    #[test]
    fn api_versioning_from_path() {
        assert_eq!(ApiVersioning::extract_version("/v1/users", None), 1);
        assert_eq!(ApiVersioning::extract_version("/v2/users", None), 2);
        assert_eq!(ApiVersioning::extract_version("/v3/users", None), 3);
    }

    #[test]
    fn api_versioning_default() {
        assert_eq!(ApiVersioning::extract_version("/users", None), 1);
    }

    #[test]
    fn api_versioning_supported_versions() {
        assert!(ApiVersioning::is_supported(1));
        assert!(ApiVersioning::is_supported(2));
        assert!(!ApiVersioning::is_supported(0));
        assert!(!ApiVersioning::is_supported(3));
    }

    #[test]
    fn api_versioning_deprecation_status() {
        assert_eq!(ApiVersioning::deprecation_status(1), "supported");
        assert_eq!(ApiVersioning::deprecation_status(2), "current");
        assert_eq!(ApiVersioning::deprecation_status(3), "unsupported");
    }

    #[test]
    fn request_timer_measures_elapsed() {
        let timer = RequestTimer::new();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let ms = timer.elapsed_ms();
        assert!(ms >= 10);
    }
}
