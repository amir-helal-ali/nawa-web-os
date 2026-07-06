//! Lock-free single-producer single-consumer ring buffer.
//!
//! Used for hot-path message passing between the kernel pipeline
//! and the worker pool. No mutexes, no allocations in the hot path.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// A bounded lock-free SPMC ring buffer.
///
/// Single producer, multi consumer. The producer pushes entries
/// atomically; consumers pop atomically. Capacity is fixed at
/// construction.
pub struct LockFreeRing<T: Clone + Send + 'static> {
    buffer: Box<[std::sync::Mutex<Option<T>>]>,
    capacity: usize,
    head: AtomicU64, // write position
    tail: AtomicU64, // read position
}

impl<T: Clone + Send + 'static> LockFreeRing<T> {
    pub fn new(capacity: usize) -> Arc<Self> {
        let mut buffer = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            buffer.push(std::sync::Mutex::new(None));
        }
        Arc::new(Self {
            buffer: buffer.into_boxed_slice(),
            capacity,
            head: AtomicU64::new(0),
            tail: AtomicU64::new(0),
        })
    }

    /// Push an entry. Returns `Err` if the ring is full.
    pub fn push(&self, item: T) -> Result<(), T> {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Acquire);
        if head.wrapping_sub(tail) >= self.capacity as u64 {
            return Err(item);
        }
        let idx = (head % self.capacity as u64) as usize;
        *self.buffer[idx].lock().unwrap() = Some(item);
        self.head.store(head.wrapping_add(1), Ordering::Release);
        Ok(())
    }

    /// Pop an entry. Returns `None` if the ring is empty.
    pub fn pop(&self) -> Option<T> {
        let tail = self.tail.load(Ordering::Relaxed);
        let head = self.head.load(Ordering::Acquire);
        if tail >= head {
            return None;
        }
        let idx = (tail % self.capacity as u64) as usize;
        let item = self.buffer[idx].lock().unwrap().take();
        self.tail.store(tail.wrapping_add(1), Ordering::Release);
        item
    }

    /// Current number of items in the ring.
    pub fn len(&self) -> usize {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Relaxed);
        head.wrapping_sub(tail) as usize
    }

    /// Is the ring empty?
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Total capacity.
    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_pop_basic() {
        let ring: Arc<LockFreeRing<u64>> = LockFreeRing::new(8);
        assert!(ring.is_empty());
        ring.push(42).unwrap();
        ring.push(99).unwrap();
        assert_eq!(ring.len(), 2);
        assert_eq!(ring.pop(), Some(42));
        assert_eq!(ring.pop(), Some(99));
        assert!(ring.is_empty());
    }

    #[test]
    fn capacity_limit() {
        let ring: Arc<LockFreeRing<u64>> = LockFreeRing::new(2);
        ring.push(1).unwrap();
        ring.push(2).unwrap();
        assert!(ring.push(3).is_err());
        assert_eq!(ring.len(), 2);
        assert_eq!(ring.capacity(), 2);
    }

    #[test]
    fn ring_wrap_around() {
        let ring: Arc<LockFreeRing<u64>> = LockFreeRing::new(2);
        for i in 0..10u64 {
            // Pop any existing items first to make room.
            while ring.pop().is_some() {}
            ring.push(i).unwrap();
            assert_eq!(ring.pop(), Some(i));
        }
    }
}
