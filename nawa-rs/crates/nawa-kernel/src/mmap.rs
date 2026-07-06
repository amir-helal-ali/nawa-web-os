//! Memory-mapped file abstraction.
//!
//! Provides zero-copy access to file contents via the kernel's page cache.
//! Uses `mmap(2)` under the hood — pages are loaded on demand and shared
//! between userspace and the kernel.

use crate::{KernelError, Result};
use memmap2::{Mmap, MmapOptions};
use std::fs::File;
use std::path::Path;

/// A memory-mapped file. The mapping is read-only by default.
///
/// # Safety
///
/// The underlying memory is owned by the kernel. As long as the file
/// is not modified externally while this mapping is alive, reads are safe.
/// We mark the mapping as `MAP_PRIVATE` so writes (if any) are copy-on-write.
pub struct MmapFile {
    _file: File,
    mmap: Mmap,
}

impl MmapFile {
    /// Open a file and map it into memory.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path.as_ref()).map_err(KernelError::Io)?;
        let mmap = unsafe { MmapOptions::new().map(&file) }
            .map_err(|e| KernelError::Mmap(e.to_string()))?;
        Ok(Self { _file: file, mmap })
    }

    /// Returns the mapped bytes. Zero-copy: no allocation.
    pub fn as_bytes(&self) -> &[u8] {
        &self.mmap[..]
    }

    /// Length of the mapped region.
    pub fn len(&self) -> usize {
        self.mmap.len()
    }

    /// Is the mapping empty?
    pub fn is_empty(&self) -> bool {
        self.mmap.is_empty()
    }

    /// Get a slice of the mapping.
    pub fn slice(&self, offset: usize, len: usize) -> Option<&[u8]> {
        let end = offset.checked_add(len)?;
        if end > self.len() {
            return None;
        }
        Some(&self.mmap[offset..end])
    }

    /// Find a byte pattern in the mapping. Returns the offset if found.
    pub fn find(&self, needle: &[u8]) -> Option<usize> {
        if needle.is_empty() || needle.len() > self.len() {
            return None;
        }
        let last_start = self.len() - needle.len();
        for i in 0..=last_start {
            if &self.mmap[i..i + needle.len()] == needle {
                return Some(i);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn mmap_and_read() {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(b"Hello, NAWA!").unwrap();
        tmp.flush().unwrap();

        let mapped = MmapFile::open(tmp.path()).unwrap();
        assert_eq!(mapped.as_bytes(), b"Hello, NAWA!");
        assert_eq!(mapped.len(), 12);
        assert!(!mapped.is_empty());
    }

    #[test]
    fn slice_and_find() {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(b"NAWA-DB::user:1001::END").unwrap();
        tmp.flush().unwrap();

        let mapped = MmapFile::open(tmp.path()).unwrap();
        assert_eq!(mapped.slice(0, 7), Some(b"NAWA-DB".as_ref()));
        assert_eq!(mapped.find(b"user:1001"), Some(9));
        assert_eq!(mapped.find(b"missing"), None);
    }
}
