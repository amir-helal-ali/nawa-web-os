//! Prometheus metrics for NAWA.
//!
//! Exposes metrics in Prometheus text format at /metrics.
//! Tracks: DB ops, io_uring stats, HTTP requests, WASM plugins.

use prometheus::{Encoder, IntCounter, IntGauge, Opts, Registry, TextEncoder};
use std::sync::Arc;

/// Container for all NAWA metrics.
pub struct Metrics {
    pub registry: Registry,

    // Database metrics
    pub db_puts: IntCounter,
    pub db_gets: IntCounter,
    pub db_deletes: IntCounter,
    pub db_scans: IntCounter,
    pub db_keys: IntGauge,
    pub db_memtable_bytes: IntGauge,
    pub db_flushes: IntCounter,

    // io_uring metrics
    pub uring_submitted: IntCounter,
    pub uring_completed: IntCounter,
    pub uring_in_flight: IntGauge,
    pub uring_errors: IntCounter,

    // HTTP metrics
    #[allow(dead_code)]
    pub http_requests: IntCounter,
    #[allow(dead_code)]
    pub http_errors: IntCounter,

    // WASM metrics
    #[allow(dead_code)]
    pub wasm_plugins_loaded: IntGauge,
    #[allow(dead_code)]
    pub wasm_invocations: IntCounter,
}

impl Metrics {
    /// Create a new metrics container with all counters registered.
    pub fn new() -> Self {
        let registry = Registry::new();

        let db_puts = IntCounter::with_opts(Opts::new("nawa_db_puts_total", "Total DB put operations")).unwrap();
        let db_gets = IntCounter::with_opts(Opts::new("nawa_db_gets_total", "Total DB get operations")).unwrap();
        let db_deletes = IntCounter::with_opts(Opts::new("nawa_db_deletes_total", "Total DB delete operations")).unwrap();
        let db_scans = IntCounter::with_opts(Opts::new("nawa_db_scans_total", "Total DB scan operations")).unwrap();
        let db_keys = IntGauge::with_opts(Opts::new("nawa_db_keys", "Current number of keys in DB")).unwrap();
        let db_memtable_bytes = IntGauge::with_opts(Opts::new("nawa_db_memtable_bytes", "Current MemTable size in bytes")).unwrap();
        let db_flushes = IntCounter::with_opts(Opts::new("nawa_db_flushes_total", "Total MemTable flushes")).unwrap();

        let uring_submitted = IntCounter::with_opts(Opts::new("nawa_uring_submitted_total", "Total io_uring submissions")).unwrap();
        let uring_completed = IntCounter::with_opts(Opts::new("nawa_uring_completed_total", "Total io_uring completions")).unwrap();
        let uring_in_flight = IntGauge::with_opts(Opts::new("nawa_uring_in_flight", "Current in-flight io_uring ops")).unwrap();
        let uring_errors = IntCounter::with_opts(Opts::new("nawa_uring_errors_total", "Total io_uring errors")).unwrap();

        let http_requests = IntCounter::with_opts(Opts::new("nawa_http_requests_total", "Total HTTP requests")).unwrap();
        let http_errors = IntCounter::with_opts(Opts::new("nawa_http_errors_total", "Total HTTP errors (5xx)")).unwrap();

        let wasm_plugins_loaded = IntGauge::with_opts(Opts::new("nawa_wasm_plugins_loaded", "Current loaded WASM plugins")).unwrap();
        let wasm_invocations = IntCounter::with_opts(Opts::new("nawa_wasm_invocations_total", "Total WASM plugin invocations")).unwrap();

        // Register all metrics.
        registry.register(Box::new(db_puts.clone())).unwrap();
        registry.register(Box::new(db_gets.clone())).unwrap();
        registry.register(Box::new(db_deletes.clone())).unwrap();
        registry.register(Box::new(db_scans.clone())).unwrap();
        registry.register(Box::new(db_keys.clone())).unwrap();
        registry.register(Box::new(db_memtable_bytes.clone())).unwrap();
        registry.register(Box::new(db_flushes.clone())).unwrap();
        registry.register(Box::new(uring_submitted.clone())).unwrap();
        registry.register(Box::new(uring_completed.clone())).unwrap();
        registry.register(Box::new(uring_in_flight.clone())).unwrap();
        registry.register(Box::new(uring_errors.clone())).unwrap();
        registry.register(Box::new(http_requests.clone())).unwrap();
        registry.register(Box::new(http_errors.clone())).unwrap();
        registry.register(Box::new(wasm_plugins_loaded.clone())).unwrap();
        registry.register(Box::new(wasm_invocations.clone())).unwrap();

        Self {
            registry,
            db_puts,
            db_gets,
            db_deletes,
            db_scans,
            db_keys,
            db_memtable_bytes,
            db_flushes,
            uring_submitted,
            uring_completed,
            uring_in_flight,
            uring_errors,
            http_requests,
            http_errors,
            wasm_plugins_loaded,
            wasm_invocations,
        }
    }

    /// Render metrics in Prometheus text format.
    pub fn render(&self) -> String {
        let mut buf = Vec::new();
        let encoder = TextEncoder::new();
        let mfs = self.registry.gather();
        encoder.encode(&mfs, &mut buf).unwrap();
        String::from_utf8(buf).unwrap()
    }

    /// Update metrics from DB stats.
    pub fn update_db_stats(&self, stats: &nawa_db::StatsSnapshot) {
        self.db_puts.inc_by(stats.puts - self.db_puts.get());
        self.db_gets.inc_by(stats.gets - self.db_gets.get());
        self.db_deletes.inc_by(stats.deletes - self.db_deletes.get());
        self.db_scans.inc_by(stats.scans - self.db_scans.get());
        self.db_flushes.inc_by(stats.memtable_flushes - self.db_flushes.get());
    }

    /// Update DB gauge metrics (current state, not counters).
    pub fn update_db_gauges(&self, keys: usize, memtable_bytes: usize) {
        self.db_keys.set(keys as i64);
        self.db_memtable_bytes.set(memtable_bytes as i64);
    }

    /// Update metrics from io_uring stats.
    pub fn update_uring_stats(&self, stats: &nawa_uring::PipelineStatsSnapshot) {
        self.uring_submitted.inc_by(stats.submitted - self.uring_submitted.get());
        self.uring_completed.inc_by(stats.completed - self.uring_completed.get());
        self.uring_in_flight.set(stats.in_flight as i64);
        self.uring_errors.inc_by(stats.errors - self.uring_errors.get());
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
pub type SharedMetrics = Arc<Metrics>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metrics_creation() {
        let m = Metrics::new();
        assert_eq!(m.db_puts.get(), 0);
        assert_eq!(m.db_keys.get(), 0);
    }

    #[test]
    fn metrics_render_non_empty() {
        let m = Metrics::new();
        m.db_puts.inc();
        m.db_keys.set(42);
        let output = m.render();
        assert!(output.contains("nawa_db_puts_total"));
        assert!(output.contains("nawa_db_keys"));
        assert!(output.contains("1"));
        assert!(output.contains("42"));
    }

    #[test]
    fn metrics_update_from_db_stats() {
        let m = Metrics::new();
        let stats = nawa_db::StatsSnapshot {
            puts: 10,
            gets: 20,
            deletes: 5,
            scans: 2,
            memtable_flushes: 1,
            bytes_written: 0,
            bytes_read: 0,
        };
        m.update_db_stats(&stats);
        assert_eq!(m.db_puts.get(), 10);
        assert_eq!(m.db_gets.get(), 20);
    }

    #[test]
    fn metrics_update_gauges() {
        let m = Metrics::new();
        m.update_db_gauges(100, 4096);
        assert_eq!(m.db_keys.get(), 100);
        assert_eq!(m.db_memtable_bytes.get(), 4096);
    }
}
