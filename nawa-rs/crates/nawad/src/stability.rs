//! Stability enhancements — connection pooling, health checks, error recovery.
//!
//! This module provides production-grade stability features:
//! - **Connection pool** for WebSocket connections with automatic cleanup
//! - **Health checker** that monitors subsystem health
//! - **Error recovery** with automatic retry logic
//! - **Graceful shutdown** with in-flight request draining

#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Connection pool — tracks active connections with automatic cleanup.
pub struct ConnectionPool {
    /// Active connections: connection_id → metadata.
    connections: RwLock<HashMap<String, ConnectionMeta>>,
    /// Total connections ever created (monotonic counter).
    total_created: AtomicU64,
    /// Total connections closed.
    total_closed: AtomicU64,
}

/// Metadata for a single connection.
#[derive(Debug, Clone)]
pub struct ConnectionMeta {
    pub id: String,
    pub remote_addr: String,
    pub connected_at: Instant,
    pub last_activity: Instant,
    pub bytes_sent: u64,
    pub bytes_received: u64,
}

impl ConnectionPool {
    /// Create a new connection pool.
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            connections: RwLock::new(HashMap::new()),
            total_created: AtomicU64::new(0),
            total_closed: AtomicU64::new(0),
        })
    }

    /// Register a new connection.
    pub async fn add(&self, id: String, remote_addr: String) {
        let now = Instant::now();
        let meta = ConnectionMeta {
            id: id.clone(),
            remote_addr,
            connected_at: now,
            last_activity: now,
            bytes_sent: 0,
            bytes_received: 0,
        };
        self.connections.write().await.insert(id, meta);
        self.total_created.fetch_add(1, Ordering::Relaxed);
    }

    /// Remove a connection.
    pub async fn remove(&self, id: &str) {
        if self.connections.write().await.remove(id).is_some() {
            self.total_closed.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Update activity timestamp for a connection.
    pub async fn touch(&self, id: &str, bytes_sent: u64, bytes_received: u64) {
        if let Some(meta) = self.connections.write().await.get_mut(id) {
            meta.last_activity = Instant::now();
            meta.bytes_sent += bytes_sent;
            meta.bytes_received += bytes_received;
        }
    }

    /// Number of active connections.
    pub async fn active_count(&self) -> usize {
        self.connections.read().await.len()
    }

    /// Total connections ever created.
    pub fn total_created(&self) -> u64 {
        self.total_created.load(Ordering::Relaxed)
    }

    /// Total connections closed.
    pub fn total_closed(&self) -> u64 {
        self.total_closed.load(Ordering::Relaxed)
    }

    /// Clean up idle connections (older than max_idle).
    pub async fn cleanup_idle(&self, max_idle: Duration) -> usize {
        let now = Instant::now();
        let mut to_remove = Vec::new();
        {
            let conns = self.connections.read().await;
            for (id, meta) in conns.iter() {
                if now.duration_since(meta.last_activity) > max_idle {
                    to_remove.push(id.clone());
                }
            }
        }
        let count = to_remove.len();
        for id in to_remove {
            self.remove(&id).await;
        }
        count
    }

    /// Get stats about the connection pool.
    pub async fn stats(&self) -> PoolStats {
        let conns = self.connections.read().await;
        let total_bytes_sent: u64 = conns.values().map(|m| m.bytes_sent).sum();
        let total_bytes_received: u64 = conns.values().map(|m| m.bytes_received).sum();
        PoolStats {
            active: conns.len(),
            total_created: self.total_created(),
            total_closed: self.total_closed(),
            total_bytes_sent,
            total_bytes_received,
        }
    }
}

/// Connection pool statistics.
#[derive(Debug, Clone, serde::Serialize)]
pub struct PoolStats {
    pub active: usize,
    pub total_created: u64,
    pub total_closed: u64,
    pub total_bytes_sent: u64,
    pub total_bytes_received: u64,
}

/// Health checker — monitors subsystem health.
pub struct HealthChecker {
    checks: RwLock<HashMap<String, HealthCheck>>,
}

/// Result of a health check.
#[derive(Debug, Clone, serde::Serialize)]
pub struct HealthCheck {
    pub name: String,
    pub healthy: bool,
    pub latency_ms: u64,
    pub message: String,
    pub checked_at: String,
}

impl HealthChecker {
    /// Create a new health checker.
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            checks: RwLock::new(HashMap::new()),
        })
    }

    /// Run a health check on a subsystem.
    pub async fn check(&self, name: &str, check_fn: impl Fn() -> bool + Send + 'static) -> HealthCheck {
        let start = Instant::now();
        let healthy = check_fn();
        let latency_ms = start.elapsed().as_millis() as u64;
        let result = HealthCheck {
            name: name.to_string(),
            healthy,
            latency_ms,
            message: if healthy { "OK".into() } else { "UNHEALTHY".into() },
            checked_at: chrono::Utc::now().to_rfc3339(),
        };
        self.checks.write().await.insert(name.to_string(), result.clone());
        result
    }

    /// Get all health check results.
    pub async fn all(&self) -> Vec<HealthCheck> {
        self.checks.read().await.values().cloned().collect()
    }

    /// Overall system health (true if all checks pass).
    pub async fn overall_healthy(&self) -> bool {
        self.checks.read().await.values().all(|c| c.healthy)
    }
}

/// Retry logic with exponential backoff.
pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(5),
            multiplier: 2.0,
        }
    }
}

/// Execute a function with retry logic.
pub async fn with_retry<F, T, E>(config: &RetryConfig, mut f: F) -> Result<T, E>
where
    F: FnMut() -> Result<T, E>,
    E: std::fmt::Display,
{
    let mut delay = config.initial_delay;
    let mut last_error = None;
    for attempt in 1..=config.max_attempts {
        match f() {
            Ok(result) => return Ok(result),
            Err(e) => {
                last_error = Some(e);
                if attempt < config.max_attempts {
                    tracing::warn!("Attempt {} failed, retrying in {:?}...", attempt, delay);
                    tokio::time::sleep(delay).await;
                    delay = Duration::from_millis(
                        (delay.as_millis() as f64 * config.multiplier) as u64
                    ).min(config.max_delay);
                }
            }
        }
    }
    Err(last_error.unwrap())
}

/// Graceful shutdown manager — drains in-flight requests before shutting down.
pub struct GracefulShutdown {
    active_requests: AtomicU64,
    shutdown_started: std::sync::atomic::AtomicBool,
}

impl GracefulShutdown {
    pub fn new() -> Self {
        Self {
            active_requests: AtomicU64::new(0),
            shutdown_started: std::sync::atomic::AtomicBool::new(false),
        }
    }

    /// Mark a request as started.
    pub fn request_started(&self) {
        self.active_requests.fetch_add(1, Ordering::SeqCst);
    }

    /// Mark a request as completed.
    pub fn request_completed(&self) {
        self.active_requests.fetch_sub(1, Ordering::SeqCst);
    }

    /// Initiate shutdown — waits for in-flight requests to complete.
    pub async fn shutdown(&self, timeout: Duration) -> bool {
        self.shutdown_started.store(true, Ordering::SeqCst);
        let start = Instant::now();
        while self.active_requests.load(Ordering::SeqCst) > 0 {
            if start.elapsed() > timeout {
                tracing::warn!("Shutdown timeout reached, {} requests still active",
                    self.active_requests.load(Ordering::SeqCst));
                return false;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        true
    }

    /// Is shutdown in progress?
    pub fn is_shutting_down(&self) -> bool {
        self.shutdown_started.load(Ordering::SeqCst)
    }

    /// Number of active requests.
    pub fn active_requests(&self) -> u64 {
        self.active_requests.load(Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn connection_pool_tracks_connections() {
        let pool = ConnectionPool::new();
        pool.add("conn1".into(), "127.0.0.1:1234".into()).await;
        pool.add("conn2".into(), "127.0.0.1:5678".into()).await;
        assert_eq!(pool.active_count().await, 2);
        assert_eq!(pool.total_created(), 2);

        pool.remove("conn1").await;
        assert_eq!(pool.active_count().await, 1);
        assert_eq!(pool.total_closed(), 1);
    }

    #[tokio::test]
    async fn connection_pool_tracks_bytes() {
        let pool = ConnectionPool::new();
        pool.add("conn1".into(), "127.0.0.1:1234".into()).await;
        pool.touch("conn1", 100, 50).await;
        pool.touch("conn1", 200, 30).await;
        let stats = pool.stats().await;
        assert_eq!(stats.total_bytes_sent, 300);
        assert_eq!(stats.total_bytes_received, 80);
    }

    #[tokio::test]
    async fn connection_pool_cleanup_idle() {
        let pool = ConnectionPool::new();
        pool.add("conn1".into(), "127.0.0.1:1234".into()).await;
        // Wait a tiny bit so the connection becomes "idle".
        tokio::time::sleep(Duration::from_millis(10)).await;
        let removed = pool.cleanup_idle(Duration::from_millis(5)).await;
        assert_eq!(removed, 1);
        assert_eq!(pool.active_count().await, 0);
    }

    #[tokio::test]
    async fn health_checker_runs_checks() {
        let checker = HealthChecker::new();
        let result = checker.check("db", || true).await;
        assert!(result.healthy);
        assert!(checker.overall_healthy().await);

        let result = checker.check("redis", || false).await;
        assert!(!result.healthy);
        assert!(!checker.overall_healthy().await);
    }

    #[tokio::test]
    async fn retry_succeeds_on_second_attempt() {
        let config = RetryConfig {
            max_attempts: 3,
            initial_delay: Duration::from_millis(1),
            max_delay: Duration::from_millis(10),
            multiplier: 2.0,
        };
        let mut attempts = 0;
        let result: Result<i32, &str> = with_retry(&config, || {
            attempts += 1;
            if attempts < 2 { Err("failed") } else { Ok(42) }
        }).await;
        assert_eq!(result.unwrap(), 42);
        assert_eq!(attempts, 2);
    }

    #[tokio::test]
    async fn retry_exhausts_attempts() {
        let config = RetryConfig {
            max_attempts: 3,
            initial_delay: Duration::from_millis(1),
            max_delay: Duration::from_millis(10),
            multiplier: 2.0,
        };
        let result: Result<i32, &str> = with_retry(&config, || Err("always fails")).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn graceful_shutdown_waits_for_requests() {
        let shutdown = GracefulShutdown::new();
        shutdown.request_started();
        shutdown.request_started();
        assert_eq!(shutdown.active_requests(), 2);

        // Complete one request.
        shutdown.request_completed();
        assert_eq!(shutdown.active_requests(), 1);

        // Start shutdown with short timeout — should not complete yet.
        let result = tokio::time::timeout(
            Duration::from_millis(100),
            shutdown.shutdown(Duration::from_millis(50))
        ).await;
        // Should timeout or return false (request still active).
        assert!(result.is_err() || !result.unwrap());

        // Complete the last request.
        shutdown.request_completed();
        assert_eq!(shutdown.active_requests(), 0);
    }

    #[tokio::test]
    async fn graceful_shutdown_completes_when_no_requests() {
        let shutdown = GracefulShutdown::new();
        let result = shutdown.shutdown(Duration::from_millis(100)).await;
        assert!(result);
    }

    #[tokio::test]
    async fn pool_stats_serializes() {
        let pool = ConnectionPool::new();
        pool.add("conn1".into(), "127.0.0.1:1234".into()).await;
        let stats = pool.stats().await;
        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("active"));
        assert!(json.contains("total_created"));
    }
}
