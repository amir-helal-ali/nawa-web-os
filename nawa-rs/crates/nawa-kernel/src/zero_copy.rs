//! Zero-copy buffer abstraction.
//!
//! A reference-counted byte slice that can be passed between
//! subsystems without copying. Built on top of `bytes::Bytes`.

use bytes::Bytes;
use std::sync::Arc;

/// A zero-copy buffer.
///
/// Internally uses `bytes::Bytes` which is reference-counted.
/// Cloning is O(1) — only bumps a refcount.
#[derive(Clone, Debug)]
pub struct ZeroCopyBuf {
    inner: Bytes,
}

impl ZeroCopyBuf {
    /// Create from a static slice (no allocation).
    pub fn from_static(slice: &'static [u8]) -> Self {
        Self {
            inner: Bytes::from_static(slice),
        }
    }

    /// Create from a `Vec<u8>`.
    pub fn from_vec(v: Vec<u8>) -> Self {
        Self {
            inner: Bytes::from(v),
        }
    }

    /// Create from `Bytes`.
    pub fn from_bytes(b: Bytes) -> Self {
        Self { inner: b }
    }

    /// View as a byte slice.
    pub fn as_slice(&self) -> &[u8] {
        &self.inner[..]
    }

    /// Length in bytes.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Is it empty?
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Slice a sub-range. Still zero-copy (refcount bump).
    pub fn slice(&self, start: usize, end: usize) -> Self {
        Self {
            inner: self.inner.slice(start..end),
        }
    }

    /// Convert to underlying `Bytes`.
    pub fn into_bytes(self) -> Bytes {
        self.inner
    }

    /// Wrap in an Arc for sharing across threads.
    pub fn shared(self) -> Arc<Self> {
        Arc::new(self)
    }
}

impl AsRef<[u8]> for ZeroCopyBuf {
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

impl From<Vec<u8>> for ZeroCopyBuf {
    fn from(v: Vec<u8>) -> Self {
        Self::from_vec(v)
    }
}

impl From<&'static [u8]> for ZeroCopyBuf {
    fn from(s: &'static [u8]) -> Self {
        Self::from_static(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_copy_clone() {
        let buf = ZeroCopyBuf::from_static(b"hello world");
        let buf2 = buf.clone();
        // Both should point to the same data.
        assert_eq!(buf.as_slice(), buf2.as_slice());
        assert_eq!(buf.len(), 11);
    }

    #[test]
    fn slice_is_zero_copy() {
        let buf = ZeroCopyBuf::from_vec(b"hello world".to_vec());
        let sub = buf.slice(0, 5);
        assert_eq!(sub.as_slice(), b"hello");
    }
}
