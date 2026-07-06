//! Built-in middleware — rate limiting, security headers.

use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::sync::Mutex;

/// Simple IP-based rate limiter.
#[allow(dead_code)]
pub struct RateLimiter {
    /// Max requests per window.
    max_requests: u32,
    /// Time window.
    window: Duration,
    /// IP → (count, window_start).
    clients: Mutex<HashMap<String, (u32, Instant)>>,
}

impl RateLimiter {
    /// Create a new rate limiter.
    pub fn new(max_requests: u32, window: Duration) -> Self {
        Self {
            max_requests,
            window,
            clients: Mutex::new(HashMap::new()),
        }
    }

    /// Check if a request from the given IP is allowed.
    /// Returns true if allowed, false if rate-limited.
    #[allow(dead_code)]
    pub fn check(&self, ip: &str) -> bool {
        let mut clients = self.clients.lock().unwrap();
        let now = Instant::now();

        let entry = clients.entry(ip.to_string()).or_insert((0, now));

        // Reset if window expired.
        if now.duration_since(entry.1) > self.window {
            *entry = (1, now);
            return true;
        }

        entry.0 += 1;
        entry.0 <= self.max_requests
    }

    /// Get current count for an IP.
    #[allow(dead_code)]
    pub fn count(&self, ip: &str) -> u32 {
        let clients = self.clients.lock().unwrap();
        clients.get(ip).map(|(c, _)| *c).unwrap_or(0)
    }

    /// Clean up expired entries (call periodically).
    #[allow(dead_code)]
    pub fn cleanup(&self) {
        let mut clients = self.clients.lock().unwrap();
        let now = Instant::now();
        clients.retain(|_, (_, start)| now.duration_since(*start) < self.window);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_under_limit() {
        let rl = RateLimiter::new(5, Duration::from_secs(60));
        for _ in 0..5 {
            assert!(rl.check("192.168.1.1"));
        }
    }

    #[test]
    fn blocks_over_limit() {
        let rl = RateLimiter::new(3, Duration::from_secs(60));
        rl.check("1.2.3.4");
        rl.check("1.2.3.4");
        rl.check("1.2.3.4");
        assert!(!rl.check("1.2.3.4")); // 4th request blocked
    }

    #[test]
    fn different_ips_independent() {
        let rl = RateLimiter::new(2, Duration::from_secs(60));
        assert!(rl.check("1.1.1.1"));
        assert!(rl.check("2.2.2.2"));
        assert!(rl.check("1.1.1.1"));
        assert!(rl.check("2.2.2.2"));
        assert!(!rl.check("1.1.1.1")); // blocked
        assert!(!rl.check("2.2.2.2")); // blocked
    }

    #[test]
    fn count_tracking() {
        let rl = RateLimiter::new(10, Duration::from_secs(60));
        rl.check("1.1.1.1");
        rl.check("1.1.1.1");
        rl.check("1.1.1.1");
        assert_eq!(rl.count("1.1.1.1"), 3);
        assert_eq!(rl.count("2.2.2.2"), 0);
    }
}
