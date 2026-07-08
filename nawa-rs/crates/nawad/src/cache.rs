//! Caching layer — in-memory LRU cache for request responses.

#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// LRU cache entry.
struct CacheEntry {
    data: Vec<u8>,
    content_type: String,
    inserted_at: Instant,
    expires_at: Option<Instant>,
    size_bytes: usize,
}

/// LRU cache statistics.
#[derive(Debug, Clone, serde::Serialize)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub entries: usize,
    pub size_bytes: usize,
    pub max_entries: usize,
    pub max_bytes: usize,
    pub hit_rate: f64,
}

/// Thread-safe LRU cache for HTTP responses.
pub struct ResponseCache {
    entries: RwLock<HashMap<String, CacheEntry>>,
    /// Order of access (oldest first) for LRU eviction.
    access_order: RwLock<Vec<String>>,
    max_entries: usize,
    max_bytes: usize,
    hits: AtomicU64,
    misses: AtomicU64,
    evictions: AtomicU64,
    current_bytes: AtomicU64,
}

impl ResponseCache {
    /// Create a new cache with the given limits.
    pub fn new(max_entries: usize, max_bytes: usize) -> Arc<Self> {
        Arc::new(Self {
            entries: RwLock::new(HashMap::new()),
            access_order: RwLock::new(Vec::new()),
            max_entries,
            max_bytes,
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
            evictions: AtomicU64::new(0),
            current_bytes: AtomicU64::new(0),
        })
    }

    /// Get a cached response by key.
    pub async fn get(&self, key: &str) -> Option<(Vec<u8>, String)> {
        let entries = self.entries.read().await;
        if let Some(entry) = entries.get(key) {
            // Check TTL expiration.
            if let Some(expires) = entry.expires_at {
                if Instant::now() >= expires {
                    self.misses.fetch_add(1, Ordering::Relaxed);
                    return None;
                }
            }
            self.hits.fetch_add(1, Ordering::Relaxed);
            return Some((entry.data.clone(), entry.content_type.clone()));
        }
        self.misses.fetch_add(1, Ordering::Relaxed);
        None
    }

    /// Insert a response into the cache.
    pub async fn insert(&self, key: String, data: Vec<u8>, content_type: String, ttl: Option<Duration>) {
        let size_bytes = data.len();
        let now = Instant::now();
        let expires_at = ttl.map(|d| now + d);

        // Evict if exceeding limits.
        self.evict_if_needed(size_bytes).await;

        let entry = CacheEntry {
            size_bytes,
            inserted_at: now,
            expires_at,
            data,
            content_type,
        };

        let mut entries = self.entries.write().await;
        let mut order = self.access_order.write().await;

        // Remove old entry if exists.
        if let Some(old) = entries.remove(&key) {
            self.current_bytes.fetch_sub(old.size_bytes as u64, Ordering::Relaxed);
            order.retain(|k| k != &key);
        }

        self.current_bytes.fetch_add(size_bytes as u64, Ordering::Relaxed);
        entries.insert(key.clone(), entry);
        order.push(key);
    }

    /// Remove a key from the cache.
    pub async fn invalidate(&self, key: &str) {
        let mut entries = self.entries.write().await;
        let mut order = self.access_order.write().await;
        if let Some(entry) = entries.remove(key) {
            self.current_bytes.fetch_sub(entry.size_bytes as u64, Ordering::Relaxed);
        }
        order.retain(|k| k != key);
    }

    /// Clear all cached entries.
    pub async fn clear(&self) {
        let mut entries = self.entries.write().await;
        let mut order = self.access_order.write().await;
        entries.clear();
        order.clear();
        self.current_bytes.store(0, Ordering::Relaxed);
    }

    /// Get cache statistics.
    pub async fn stats(&self) -> CacheStats {
        let entries = self.entries.read().await;
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let total = hits + misses;
        let hit_rate = if total > 0 {
            (hits as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        CacheStats {
            hits,
            misses,
            evictions: self.evictions.load(Ordering::Relaxed),
            entries: entries.len(),
            size_bytes: self.current_bytes.load(Ordering::Relaxed) as usize,
            max_entries: self.max_entries,
            max_bytes: self.max_bytes,
            hit_rate,
        }
    }

    /// Evict entries if exceeding size or count limits.
    async fn evict_if_needed(&self, incoming_size: usize) {
        loop {
            let current_entries = self.entries.read().await.len();
            let current_bytes = self.current_bytes.load(Ordering::Relaxed) as usize;

            // Check if we need to evict.
            let need_evict = current_entries >= self.max_entries
                || current_bytes + incoming_size > self.max_bytes;

            if !need_evict {
                break;
            }

            // Evict the oldest entry (LRU).
            let oldest_key = {
                let mut order = self.access_order.write().await;
                if order.is_empty() {
                    break;
                }
                order.remove(0)
            };

            let mut entries = self.entries.write().await;
            if let Some(entry) = entries.remove(&oldest_key) {
                self.current_bytes.fetch_sub(entry.size_bytes as u64, Ordering::Relaxed);
                self.evictions.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
}

/// Cache key generator for HTTP requests.
pub fn cache_key(method: &str, path: &str, query: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    method.hash(&mut hasher);
    path.hash(&mut hasher);
    query.hash(&mut hasher);
    format!("cache:{:016x}", hasher.finish())
}

/// Cache-aside helper: get from cache or compute and store.
pub async fn get_or_compute<F, Fut>(
    cache: &ResponseCache,
    key: &str,
    ttl: Option<Duration>,
    compute: F,
) -> (Vec<u8>, String)
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = (Vec<u8>, String)>,
{
    if let Some((data, ct)) = cache.get(key).await {
        return (data, ct);
    }
    let (data, ct) = compute().await;
    cache.insert(key.to_string(), data.clone(), ct.clone(), ttl).await;
    (data, ct)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn cache_insert_and_get() {
        let cache = ResponseCache::new(100, 1024 * 1024);
        cache.insert("key1".into(), b"value1".to_vec(), "text/plain".into(), None).await;
        let result = cache.get("key1").await;
        assert!(result.is_some());
        let (data, ct) = result.unwrap();
        assert_eq!(data, b"value1");
        assert_eq!(ct, "text/plain");
    }

    #[tokio::test]
    async fn cache_miss() {
        let cache = ResponseCache::new(100, 1024 * 1024);
        let result = cache.get("nonexistent").await;
        assert!(result.is_none());
        let stats = cache.stats().await;
        assert_eq!(stats.misses, 1);
    }

    #[tokio::test]
    async fn cache_hit_increments_counter() {
        let cache = ResponseCache::new(100, 1024 * 1024);
        cache.insert("key1".into(), b"value".to_vec(), "text/plain".into(), None).await;
        cache.get("key1").await;
        cache.get("key1").await;
        let stats = cache.stats().await;
        assert_eq!(stats.hits, 2);
    }

    #[tokio::test]
    async fn cache_ttl_expiration() {
        let cache = ResponseCache::new(100, 1024 * 1024);
        cache.insert("key1".into(), b"value".to_vec(), "text/plain".into(),
            Some(Duration::from_millis(10))).await;
        // Should be available immediately.
        assert!(cache.get("key1").await.is_some());
        // Wait for expiration.
        tokio::time::sleep(Duration::from_millis(20)).await;
        // Should be expired (miss).
        assert!(cache.get("key1").await.is_none());
    }

    #[tokio::test]
    async fn cache_eviction_on_max_entries() {
        let cache = ResponseCache::new(3, 1024 * 1024);
        cache.insert("k1".into(), b"v1".to_vec(), "text/plain".into(), None).await;
        cache.insert("k2".into(), b"v2".to_vec(), "text/plain".into(), None).await;
        cache.insert("k3".into(), b"v3".to_vec(), "text/plain".into(), None).await;
        cache.insert("k4".into(), b"v4".to_vec(), "text/plain".into(), None).await;
        let stats = cache.stats().await;
        assert!(stats.entries <= 3);
        assert!(stats.evictions >= 1);
    }

    #[tokio::test]
    async fn cache_invalidate() {
        let cache = ResponseCache::new(100, 1024 * 1024);
        cache.insert("key1".into(), b"value".to_vec(), "text/plain".into(), None).await;
        assert!(cache.get("key1").await.is_some());
        cache.invalidate("key1").await;
        assert!(cache.get("key1").await.is_none());
    }

    #[tokio::test]
    async fn cache_clear() {
        let cache = ResponseCache::new(100, 1024 * 1024);
        cache.insert("k1".into(), b"v1".to_vec(), "text/plain".into(), None).await;
        cache.insert("k2".into(), b"v2".to_vec(), "text/plain".into(), None).await;
        cache.clear().await;
        let stats = cache.stats().await;
        assert_eq!(stats.entries, 0);
    }

    #[tokio::test]
    async fn cache_stats_hit_rate() {
        let cache = ResponseCache::new(100, 1024 * 1024);
        cache.insert("k1".into(), b"v1".to_vec(), "text/plain".into(), None).await;
        cache.get("k1").await; // hit
        cache.get("k1").await; // hit
        cache.get("k2").await; // miss
        let stats = cache.stats().await;
        assert_eq!(stats.hits, 2);
        assert_eq!(stats.misses, 1);
        assert!((stats.hit_rate - 66.67).abs() < 0.1);
    }

    #[tokio::test]
    async fn get_or_compute_caches_result() {
        let cache = ResponseCache::new(100, 1024 * 1024);
        let call_count = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let cc = call_count.clone();
        let (data1, _) = get_or_compute(&cache, "key1", None, || {
            let cc = cc.clone();
            async move {
                cc.fetch_add(1, Ordering::Relaxed);
                (b"computed".to_vec(), "text/plain".into())
            }
        }).await;
        assert_eq!(data1, b"computed");
        // Second call should use cache (compute not called).
        let cc2 = call_count.clone();
        let (data2, _) = get_or_compute(&cache, "key1", None, || {
            let cc = cc2.clone();
            async move {
                cc.fetch_add(1, Ordering::Relaxed);
                (b"computed_again".to_vec(), "text/plain".into())
            }
        }).await;
        assert_eq!(data2, b"computed");
        assert_eq!(call_count.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn cache_key_is_deterministic() {
        let k1 = cache_key("GET", "/users/42", "");
        let k2 = cache_key("GET", "/users/42", "");
        assert_eq!(k1, k2);
        let k3 = cache_key("GET", "/users/43", "");
        assert_ne!(k1, k3);
    }
}
