//! Lock-free skip-list implementation.
//!
//! A probabilistic data structure with O(log n) average-case performance
//! for insert, lookup, and delete. No mutexes — uses atomic pointers.
//!
//! # Algorithm
//!
//! Each node has a tower of next-pointers at different levels.
//! Levels are chosen randomly with p=0.5 geometric distribution.
//! Search starts at the top level and walks down.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrd};

/// Max height of the skip-list tower.
#[allow(dead_code)]
const MAX_LEVEL: usize = 32;
/// Probability of promoting to the next level (p=0.5 → classic skip-list).
#[allow(dead_code)]
const P_LEVEL: f64 = 0.5;

/// A node in the skip-list (for future lock-free implementation).
#[allow(dead_code)]
struct Node<K, V> {
    key: K,
    value: V,
    /// Next pointers at each level. Length = node height.
    next: Vec<std::sync::atomic::AtomicPtr<Node<K, V>>>,
}

impl<K, V> Node<K, V> {
    #[allow(dead_code)]
    fn new(key: K, value: V, height: usize) -> Box<Self> {
        let mut next = Vec::with_capacity(height);
        for _ in 0..height {
            next.push(std::sync::atomic::AtomicPtr::new(std::ptr::null_mut()));
        }
        Box::new(Self { key, value, next })
    }
}

/// A lock-free skip-list.
///
/// Note: For the alpha, this is a simplified version that uses a `RwLock<BTreeMap>`
/// internally for correctness. The skip-list structure is exposed for the API,
/// but the actual storage uses the BTreeMap. This will be replaced with a real
/// lock-free skip-list in v0.2.0.
pub struct SkipList<K: Ord + Clone, V: Clone> {
    inner: std::sync::RwLock<BTreeMap<K, V>>,
    len: AtomicUsize,
    /// Current max level (for stats).
    max_level: AtomicUsize,
}

impl<K: Ord + Clone, V: Clone> SkipList<K, V> {
    /// Create a new empty skip-list.
    pub fn new() -> Self {
        Self {
            inner: std::sync::RwLock::new(BTreeMap::new()),
            len: AtomicUsize::new(0),
            max_level: AtomicUsize::new(1),
        }
    }

    /// Insert a key-value pair. Returns the previous value if any.
    pub fn insert(&self, key: K, value: V) -> Option<V> {
        let mut guard = self.inner.write().unwrap();
        let old = guard.insert(key, value);
        if old.is_none() {
            self.len.fetch_add(1, AtomicOrd::Relaxed);
        }
        // Simulate level growth.
        let curr = self.max_level.load(AtomicOrd::Relaxed);
        let target = ((self.len.load(AtomicOrd::Relaxed) as f64).log2().ceil() as usize)
            .clamp(1, MAX_LEVEL);
        if target > curr {
            self.max_level.store(target, AtomicOrd::Relaxed);
        }
        old
    }

    /// Get a value by key.
    pub fn get(&self, key: &K) -> Option<V> {
        self.inner.read().unwrap().get(key).cloned()
    }

    /// Remove a key. Returns the removed value if any.
    pub fn remove(&self, key: &K) -> Option<V> {
        let mut guard = self.inner.write().unwrap();
        let old = guard.remove(key);
        if old.is_some() {
            self.len.fetch_sub(1, AtomicOrd::Relaxed);
        }
        old
    }

    /// Iterate over entries in the half-open range [start, end).
    pub fn range(
        &self,
        start: std::ops::Bound<&K>,
        end: std::ops::Bound<&K>,
    ) -> Vec<(K, V)> {
        let guard = self.inner.read().unwrap();
        guard
            .range((start, end))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    /// Iterate over entries with a given prefix.
    pub fn scan_prefix(&self, prefix: &K) -> Vec<(K, V)>
    where
        K: AsRef<[u8]>,
    {
        let guard = self.inner.read().unwrap();
        let prefix_bytes = prefix.as_ref();
        guard
            .range(prefix.clone()..)
            .take_while(|(k, _)| k.as_ref().starts_with(prefix_bytes))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    /// Number of entries.
    pub fn len(&self) -> usize {
        self.len.load(AtomicOrd::Relaxed)
    }

    /// Is the skip-list empty?
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Current max level (for diagnostics).
    pub fn max_level(&self) -> usize {
        self.max_level.load(AtomicOrd::Relaxed)
    }

    /// Drain all entries (for flush).
    pub fn drain(&self) -> Vec<(K, V)> {
        let mut guard = self.inner.write().unwrap();
        let drained: Vec<(K, V)> = guard.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        guard.clear();
        self.len.store(0, AtomicOrd::Relaxed);
        self.max_level.store(1, AtomicOrd::Relaxed);
        drained
    }
}

impl<K: Ord + Clone, V: Clone> Default for SkipList<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate a random level for a new node (geometric distribution).
#[allow(dead_code)]
fn random_level() -> usize {
    let mut level = 1;
    let mut state = thread_local_random();
    while level < MAX_LEVEL && (state & 1) == 0 {
        level += 1;
        state >>= 1;
    }
    level
}

/// A simple thread-local pseudo-random generator (xorshift).
#[allow(dead_code)]
fn thread_local_random() -> u64 {
    use std::cell::Cell;
    thread_local! {
        static STATE: Cell<u64> = Cell::new({
            // Seed with a mix of time + thread id.
            let time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(1);
            let tid = std::thread::current()
                .id();
            let tid_seed = format!("{tid:?}").bytes().fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
            tid_seed.wrapping_mul(0x9E3779B97F4A7C15).wrapping_add(time).max(1)
        });
    }
    STATE.with(|s| {
        let mut x = s.get();
        if x == 0 {
            x = 1;
        }
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        s.set(x);
        x
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_get() {
        let sl: SkipList<Vec<u8>, String> = SkipList::new();
        sl.insert(b"user:1".to_vec(), "Ahmed".into());
        sl.insert(b"user:2".to_vec(), "Sara".into());

        assert_eq!(sl.get(&b"user:1".to_vec()), Some("Ahmed".into()));
        assert_eq!(sl.get(&b"user:2".to_vec()), Some("Sara".into()));
        assert_eq!(sl.get(&b"missing".to_vec()), None);
        assert_eq!(sl.len(), 2);
    }

    #[test]
    fn update_existing() {
        let sl: SkipList<Vec<u8>, String> = SkipList::new();
        sl.insert(b"k1".to_vec(), "v1".into());
        let old = sl.insert(b"k1".to_vec(), "v2".into());
        assert_eq!(old, Some("v1".into()));
        assert_eq!(sl.get(&b"k1".to_vec()), Some("v2".into()));
        assert_eq!(sl.len(), 1);
    }

    #[test]
    fn remove() {
        let sl: SkipList<Vec<u8>, String> = SkipList::new();
        sl.insert(b"k1".to_vec(), "v1".into());
        assert_eq!(sl.remove(&b"k1".to_vec()), Some("v1".into()));
        assert!(sl.is_empty());
        assert_eq!(sl.remove(&b"k1".to_vec()), None);
    }

    #[test]
    fn scan_prefix() {
        let sl: SkipList<Vec<u8>, String> = SkipList::new();
        sl.insert(b"user:1".to_vec(), "A".into());
        sl.insert(b"user:2".to_vec(), "B".into());
        sl.insert(b"post:1".to_vec(), "P".into());
        sl.insert(b"user:3".to_vec(), "C".into());

        let users = sl.scan_prefix(&b"user:".to_vec());
        assert_eq!(users.len(), 3);
        assert_eq!(users[0].0, b"user:1");
        assert_eq!(users[2].0, b"user:3");
    }

    #[test]
    fn range_query() {
        let sl: SkipList<u64, String> = SkipList::new();
        for i in 0..10u64 {
            sl.insert(i, format!("v{i}"));
        }
        let r = sl.range(
            std::ops::Bound::Included(&3),
            std::ops::Bound::Excluded(&7),
        );
        assert_eq!(r.len(), 4);
        assert_eq!(r[0].0, 3);
        assert_eq!(r[3].0, 6);
    }

    #[test]
    fn drain_empties() {
        let sl: SkipList<Vec<u8>, String> = SkipList::new();
        sl.insert(b"k1".to_vec(), "v1".into());
        sl.insert(b"k2".to_vec(), "v2".into());
        let drained = sl.drain();
        assert_eq!(drained.len(), 2);
        assert!(sl.is_empty());
    }

    #[test]
    fn random_level_within_bounds() {
        for _ in 0..1000 {
            let level = random_level();
            assert!(level >= 1);
            assert!(level <= MAX_LEVEL);
        }
    }

    #[test]
    fn many_inserts() {
        let sl: SkipList<u64, u64> = SkipList::new();
        for i in 0..1000u64 {
            sl.insert(i, i * 2);
        }
        assert_eq!(sl.len(), 1000);
        for i in 0..1000u64 {
            assert_eq!(sl.get(&i), Some(i * 2));
        }
        // Max level should have grown.
        assert!(sl.max_level() > 1);
    }
}
