//! MemTable — in-memory sorted table backed by a lock-free skip-list.
//!
//! All writes go here first, before being flushed to an SSTable.

use std::collections::BTreeMap;
use std::sync::RwLock;

use crate::Value;

/// An in-memory sorted map. Writes are fast, reads are O(log n).
///
/// Note: For maximum performance in production, this would be a lock-free
/// skip-list. For the alpha, we use a `BTreeMap` behind an `RwLock` —
/// still O(log n) and good enough for the prototype.
pub struct MemTable {
    inner: RwLock<BTreeMap<Vec<u8>, Value>>,
    max_size: usize,
    current_size: std::sync::atomic::AtomicUsize,
}

impl MemTable {
    /// Create a new MemTable with a max byte size.
    pub fn new(max_size: usize) -> Self {
        Self {
            inner: RwLock::new(BTreeMap::new()),
            max_size,
            current_size: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    /// Insert a key-value pair.
    pub fn put(&self, key: Vec<u8>, value: Value) {
        let entry_size = key.len() + value.to_stored().len();
        let mut guard = self.inner.write().unwrap();
        // If updating, subtract the old size first.
        if let Some(old) = guard.get(&key) {
            let old_size = key.len() + old.to_stored().len();
            self.current_size
                .fetch_sub(old_size, std::sync::atomic::Ordering::Relaxed);
        }
        guard.insert(key, value);
        self.current_size
            .fetch_add(entry_size, std::sync::atomic::Ordering::Relaxed);
    }

    /// Get a value by key.
    pub fn get(&self, key: &[u8]) -> Option<Value> {
        let guard = self.inner.read().unwrap();
        guard.get(key).cloned()
    }

    /// Delete a key (tombstone by removing — for alpha).
    pub fn delete(&self, key: &[u8]) -> bool {
        let mut guard = self.inner.write().unwrap();
        if let Some(old) = guard.remove(key) {
            let old_size = key.len() + old.to_stored().len();
            self.current_size
                .fetch_sub(old_size, std::sync::atomic::Ordering::Relaxed);
            true
        } else {
            false
        }
    }

    /// Scan keys with a prefix. Returns up to `limit` results.
    pub fn scan_prefix(&self, prefix: &[u8], limit: usize) -> Vec<(Vec<u8>, Value)> {
        let guard = self.inner.read().unwrap();
        let prefix_vec = prefix.to_vec();
        guard
            .range(prefix_vec.clone()..)
            .take_while(|(k, _)| k.starts_with(prefix))
            .take(limit)
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    /// Count of entries.
    pub fn len(&self) -> usize {
        let guard = self.inner.read().unwrap();
        guard.len()
    }

    /// Is the table empty?
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Current byte size.
    pub fn size(&self) -> usize {
        self.current_size.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Is the table full and needs to be flushed?
    pub fn is_full(&self) -> bool {
        self.size() >= self.max_size
    }

    /// Drain all entries (used when flushing to SSTable).
    pub fn drain(&self) -> BTreeMap<Vec<u8>, Value> {
        let mut guard = self.inner.write().unwrap();
        let drained = std::mem::take(&mut *guard);
        self.current_size
            .store(0, std::sync::atomic::Ordering::Relaxed);
        drained
    }
}

impl Default for MemTable {
    fn default() -> Self {
        Self::new(4 * 1024 * 1024) // 4 MB default
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn put_get_basic() {
        let mt = MemTable::new(1024);
        mt.put(b"user:1001".to_vec(), Value::from_str("Ahmed"));
        mt.put(b"user:1002".to_vec(), Value::from_str("Sara"));

        assert_eq!(
            mt.get(b"user:1001").map(|v| v.display()),
            Some("Ahmed".into())
        );
        assert_eq!(
            mt.get(b"user:1002").map(|v| v.display()),
            Some("Sara".into())
        );
        assert!(mt.get(b"missing").is_none());
    }

    #[test]
    fn scan_prefix() {
        let mt = MemTable::new(1024);
        mt.put(b"user:1".to_vec(), Value::from_str("A"));
        mt.put(b"user:2".to_vec(), Value::from_str("B"));
        mt.put(b"post:1".to_vec(), Value::from_str("P1"));
        mt.put(b"user:3".to_vec(), Value::from_str("C"));

        let users = mt.scan_prefix(b"user:", 100);
        assert_eq!(users.len(), 3);
        assert_eq!(users[0].0, b"user:1");
        assert_eq!(users[1].0, b"user:2");
        assert_eq!(users[2].0, b"user:3");
    }

    #[test]
    fn delete_and_size() {
        let mt = MemTable::new(1024);
        mt.put(b"k1".to_vec(), Value::from_str("v1"));
        assert_eq!(mt.len(), 1);
        assert!(mt.size() > 0);

        assert!(mt.delete(b"k1"));
        assert_eq!(mt.len(), 0);
        assert_eq!(mt.size(), 0);
        assert!(!mt.delete(b"k1"));
    }

    #[test]
    fn is_full_check() {
        let mt = MemTable::new(10); // tiny capacity
        mt.put(b"12345".to_vec(), Value::from_str("12345")); // 10 bytes
        assert!(mt.is_full());
    }

    #[test]
    fn drain_empties() {
        let mt = MemTable::new(1024);
        mt.put(b"k1".to_vec(), Value::from_str("v1"));
        mt.put(b"k2".to_vec(), Value::from_str("v2"));
        let drained = mt.drain();
        assert_eq!(drained.len(), 2);
        assert!(mt.is_empty());
    }
}
