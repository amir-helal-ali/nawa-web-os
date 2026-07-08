//! Registered buffers — pre-registered memory regions for zero-copy I/O.
//!
//! When buffers are registered with io_uring, the kernel pins them in memory
//! and can perform I/O directly to/from them without page table lookups per op.
//! This eliminates overhead for high-frequency operations.

use std::sync::Arc;

/// A single registered buffer.
pub struct RegisteredBuffer {
    /// The buffer data.
    data: Vec<u8>,
    /// Index assigned by the kernel (after registration).
    index: Option<u32>,
}

impl RegisteredBuffer {
    /// Create a new buffer of the given size.
    pub fn new(size: usize) -> Self {
        Self {
            data: vec![0u8; size],
            index: None,
        }
    }

    /// Create from an existing vector.
    pub fn from_vec(data: Vec<u8>) -> Self {
        Self { data, index: None }
    }

    /// Get the buffer as a slice.
    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    /// Get the buffer as a mutable slice.
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.data
    }

    /// Get the buffer length.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Is the buffer empty?
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Has this buffer been registered with the kernel?
    pub fn is_registered(&self) -> bool {
        self.index.is_some()
    }

    /// Get the kernel-assigned index (if registered).
    pub fn index(&self) -> Option<u32> {
        self.index
    }

    /// Mark as registered with the given index (internal use).
    #[allow(dead_code)]
    pub(crate) fn set_index(&mut self, index: u32) {
        self.index = Some(index);
    }

    /// Get the buffer's physical address (for io_uring submissions).
    pub fn addr(&self) -> u64 {
        self.data.as_ptr() as u64
    }
}

/// A collection of registered buffers.
///
/// All buffers in this collection are registered with the kernel at once,
/// which is more efficient than registering them individually.
pub struct RegisteredBuffers {
    buffers: Vec<Arc<std::sync::Mutex<RegisteredBuffer>>>,
    total_size: usize,
    registered: bool,
}

impl RegisteredBuffers {
    /// Create a new empty collection.
    pub fn new() -> Self {
        Self {
            buffers: Vec::new(),
            total_size: 0,
            registered: false,
        }
    }

    /// Add a buffer of the given size.
    pub fn add_buffer(&mut self, size: usize) -> u32 {
        let index = self.buffers.len() as u32;
        self.buffers.push(Arc::new(std::sync::Mutex::new(
            RegisteredBuffer::new(size),
        )));
        self.total_size += size;
        index
    }

    /// Add an existing buffer.
    pub fn add_existing(&mut self, buf: RegisteredBuffer) -> u32 {
        let index = self.buffers.len() as u32;
        self.total_size += buf.len();
        self.buffers.push(Arc::new(std::sync::Mutex::new(buf)));
        index
    }

    /// Get a buffer by index.
    pub fn get(&self, index: u32) -> Option<Arc<std::sync::Mutex<RegisteredBuffer>>> {
        self.buffers.get(index as usize).cloned()
    }

    /// Number of buffers in the collection.
    pub fn len(&self) -> usize {
        self.buffers.len()
    }

    /// Is the collection empty?
    pub fn is_empty(&self) -> bool {
        self.buffers.is_empty()
    }

    /// Total size of all buffers in bytes.
    pub fn total_size(&self) -> usize {
        self.total_size
    }

    /// Have these buffers been registered with the kernel?
    pub fn is_registered(&self) -> bool {
        self.registered
    }

    /// Mark all buffers as registered (internal use).
    #[allow(dead_code)]
    pub(crate) fn mark_registered(&mut self) {
        for (i, buf) in self.buffers.iter().enumerate() {
            if let Ok(mut guard) = buf.lock() {
                guard.set_index(i as u32);
            }
        }
        self.registered = true;
    }

    /// Get all buffer addresses (for kernel registration).
    pub fn addresses(&self) -> Vec<(u64, usize)> {
        self.buffers
            .iter()
            .map(|buf| {
                let guard = buf.lock().unwrap();
                (guard.addr(), guard.len())
            })
            .collect()
    }
}

impl Default for RegisteredBuffers {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buffer_creation() {
        let buf = RegisteredBuffer::new(1024);
        assert_eq!(buf.len(), 1024);
        assert!(!buf.is_empty());
        assert!(!buf.is_registered());
        assert!(buf.index().is_none());
    }

    #[test]
    fn buffer_from_vec() {
        let buf = RegisteredBuffer::from_vec(vec![1, 2, 3, 4]);
        assert_eq!(buf.len(), 4);
        assert_eq!(buf.as_slice(), &[1, 2, 3, 4]);
    }

    #[test]
    fn buffer_set_index() {
        let mut buf = RegisteredBuffer::new(64);
        buf.set_index(5);
        assert!(buf.is_registered());
        assert_eq!(buf.index(), Some(5));
    }

    #[test]
    fn buffer_addr_non_null() {
        let buf = RegisteredBuffer::new(64);
        assert!(buf.addr() != 0);
    }

    #[test]
    fn collection_add_buffers() {
        let mut coll = RegisteredBuffers::new();
        assert!(coll.is_empty());

        let i0 = coll.add_buffer(1024);
        let i1 = coll.add_buffer(2048);
        let i2 = coll.add_buffer(4096);

        assert_eq!(i0, 0);
        assert_eq!(i1, 1);
        assert_eq!(i2, 2);
        assert_eq!(coll.len(), 3);
        assert_eq!(coll.total_size(), 1024 + 2048 + 4096);
        assert!(!coll.is_registered());
    }

    #[test]
    fn collection_get_buffer() {
        let mut coll = RegisteredBuffers::new();
        let idx = coll.add_buffer(512);
        let buf = coll.get(idx).unwrap();
        let guard = buf.lock().unwrap();
        assert_eq!(guard.len(), 512);
    }

    #[test]
    fn collection_mark_registered() {
        let mut coll = RegisteredBuffers::new();
        coll.add_buffer(64);
        coll.add_buffer(128);
        assert!(!coll.is_registered());

        coll.mark_registered();
        assert!(coll.is_registered());

        // All buffers should now have indices.
        for i in 0..coll.len() {
            let buf = coll.get(i as u32).unwrap();
            let guard = buf.lock().unwrap();
            assert!(guard.is_registered());
            assert_eq!(guard.index(), Some(i as u32));
        }
    }

    #[test]
    fn collection_addresses() {
        let mut coll = RegisteredBuffers::new();
        coll.add_buffer(64);
        coll.add_buffer(128);

        let addrs = coll.addresses();
        assert_eq!(addrs.len(), 2);
        assert!(addrs[0].0 != 0);
        assert_eq!(addrs[0].1, 64);
        assert_eq!(addrs[1].1, 128);
    }
}
