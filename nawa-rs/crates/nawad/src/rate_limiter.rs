//! Sliding window rate limiter — more accurate than fixed windows.

#![allow(dead_code)]

use std::collections::HashMap;
use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Sliding window rate limiter.
pub struct SlidingWindowRateLimiter {
    /// Per-client request timestamps (within the window).
    clients: RwLock<HashMap<String, VecDeque<Instant>>>,
    /// Max requests per window.
    max_requests: u32,
    /// Window duration.
    window: Duration,
    /// Total allowed requests (counter).
    total_allowed: AtomicU64,
    /// Total denied requests (counter).
    total_denied: AtomicU64,
}

impl SlidingWindowRateLimiter {
    /// Create a new sliding window rate limiter.
    pub fn new(max_requests: u32, window: Duration) -> Arc<Self> {
        Arc::new(Self {
            clients: RwLock::new(HashMap::new()),
            max_requests,
            window,
            total_allowed: AtomicU64::new(0),
            total_denied: AtomicU64::new(0),
        })
    }

    /// Check if a request from the given client is allowed.
    /// Returns `Ok(())` if allowed, `Err(retry_after_secs)` if denied.
    pub async fn check(&self, client_id: &str) -> Result<(), u64> {
        let now = Instant::now();
        let window_start = now - self.window;
        let mut clients = self.clients.write().await;

        let timestamps = clients.entry(client_id.to_string()).or_default();

        // Remove timestamps outside the window.
        while let Some(&ts) = timestamps.front() {
            if ts < window_start {
                timestamps.pop_front();
            } else {
                break;
            }
        }

        // Check if we're at the limit.
        if timestamps.len() >= self.max_requests as usize {
            self.total_denied.fetch_add(1, Ordering::Relaxed);
            // Calculate retry-after: time until the oldest request exits the window.
            let oldest = timestamps.front().unwrap();
            let retry_after = *oldest + self.window - now;
            let retry_secs = retry_after.as_secs().max(1);
            return Err(retry_secs);
        }

        // Allow the request.
        timestamps.push_back(now);
        self.total_allowed.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    /// Get statistics for the rate limiter.
    pub async fn stats(&self) -> RateLimiterStats {
        let clients = self.clients.read().await;
        let active_clients = clients.len();
        let total_requests: usize = clients.values().map(|q| q.len()).sum();
        RateLimiterStats {
            active_clients,
            total_requests_in_window: total_requests,
            total_allowed: self.total_allowed.load(Ordering::Relaxed),
            total_denied: self.total_denied.load(Ordering::Relaxed),
            max_requests: self.max_requests,
            window_secs: self.window.as_secs(),
        }
    }

    /// Clean up stale client entries (no requests in the window).
    pub async fn cleanup(&self) -> usize {
        let now = Instant::now();
        let window_start = now - self.window;
        let mut clients = self.clients.write().await;
        let before = clients.len();
        clients.retain(|_, timestamps| {
            // Keep only if there's at least one request in the window.
            timestamps.iter().any(|&ts| ts >= window_start)
        });
        before - clients.len()
    }
}

/// Rate limiter statistics.
#[derive(Debug, Clone, serde::Serialize)]
pub struct RateLimiterStats {
    pub active_clients: usize,
    pub total_requests_in_window: usize,
    pub total_allowed: u64,
    pub total_denied: u64,
    pub max_requests: u32,
    pub window_secs: u64,
}

/// Extract client ID from a request (IP-based).
pub fn extract_client_id(
    forwarded_for: Option<&str>,
    real_ip: Option<&str>,
    remote_addr: Option<&str>,
) -> String {
    // Try X-Forwarded-For first (behind proxy).
    if let Some(xff) = forwarded_for {
        if let Some(first_ip) = xff.split(',').next() {
            return first_ip.trim().to_string();
        }
    }
    // Then X-Real-IP.
    if let Some(ip) = real_ip {
        return ip.trim().to_string();
    }
    // Fall back to remote addr.
    remote_addr.unwrap_or("unknown").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn allows_requests_under_limit() {
        let limiter = SlidingWindowRateLimiter::new(5, Duration::from_secs(60));
        for _ in 0..5 {
            assert!(limiter.check("client1").await.is_ok());
        }
    }

    #[tokio::test]
    async fn denies_requests_over_limit() {
        let limiter = SlidingWindowRateLimiter::new(3, Duration::from_secs(60));
        limiter.check("client1").await.unwrap();
        limiter.check("client1").await.unwrap();
        limiter.check("client1").await.unwrap();
        let result = limiter.check("client1").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn different_clients_have_separate_limits() {
        let limiter = SlidingWindowRateLimiter::new(2, Duration::from_secs(60));
        limiter.check("client1").await.unwrap();
        limiter.check("client1").await.unwrap();
        // client2 should still be allowed.
        assert!(limiter.check("client2").await.is_ok());
    }

    #[tokio::test]
    async fn window_slides_over_time() {
        let limiter = SlidingWindowRateLimiter::new(2, Duration::from_millis(50));
        limiter.check("client1").await.unwrap();
        limiter.check("client1").await.unwrap();
        // Denied now.
        assert!(limiter.check("client1").await.is_err());
        // Wait for window to slide.
        tokio::time::sleep(Duration::from_millis(60)).await;
        // Should be allowed again.
        assert!(limiter.check("client1").await.is_ok());
    }

    #[tokio::test]
    async fn stats_track_allowed_and_denied() {
        let limiter = SlidingWindowRateLimiter::new(2, Duration::from_secs(60));
        limiter.check("c1").await.unwrap();
        limiter.check("c1").await.unwrap();
        limiter.check("c1").await.unwrap_err();
        let stats = limiter.stats().await;
        assert_eq!(stats.total_allowed, 2);
        assert_eq!(stats.total_denied, 1);
    }

    #[tokio::test]
    async fn cleanup_removes_stale_clients() {
        let limiter = SlidingWindowRateLimiter::new(1, Duration::from_millis(50));
        limiter.check("c1").await.unwrap();
        tokio::time::sleep(Duration::from_millis(60)).await;
        let removed = limiter.cleanup().await;
        assert_eq!(removed, 1);
    }

    #[test]
    fn extract_client_id_prefers_forwarded_for() {
        let id = extract_client_id(
            Some("203.0.113.5, 70.41.3.18"),
            Some("127.0.0.1"),
            Some("127.0.0.1:1234"),
        );
        assert_eq!(id, "203.0.113.5");
    }

    #[test]
    fn extract_client_id_falls_back_to_real_ip() {
        let id = extract_client_id(None, Some("203.0.113.5"), Some("127.0.0.1:1234"));
        assert_eq!(id, "203.0.113.5");
    }

    #[test]
    fn extract_client_id_uses_remote_addr() {
        let id = extract_client_id(None, None, Some("127.0.0.1:1234"));
        assert_eq!(id, "127.0.0.1:1234");
    }

    #[tokio::test]
    async fn retry_after_is_reasonable() {
        let limiter = SlidingWindowRateLimiter::new(1, Duration::from_secs(10));
        limiter.check("c1").await.unwrap();
        let retry = limiter.check("c1").await.unwrap_err();
        assert!((1..=10).contains(&retry));
    }
}
