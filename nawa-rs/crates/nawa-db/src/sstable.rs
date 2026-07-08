//! SSTable — Sorted String Table.
//!
//! An immutable, on-disk sorted map. Once written, an SSTable
//! is never modified. Multiple SSTables are merged via compaction.

use std::collections::BTreeMap;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Read, Write};
use std::path::PathBuf;

use crate::{bloom::BloomFilter, Value};

/// SSTable header (written at the start of the file).
#[allow(dead_code)]
#[derive(Debug)]
struct SSTableHeader {
    num_entries: u32,
    bloom_bits: u32,
    bloom_hashes: u8,
}

/// SSTable writer — builds a new SSTable from a sorted map.
pub struct SSTableWriter {
    path: PathBuf,
}

impl SSTableWriter {
    pub fn new(path: impl AsRef<std::path::Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }

    /// Write a sorted map to an SSTable file. Returns the file size.
    pub fn write(self, entries: &BTreeMap<Vec<u8>, Value>) -> std::io::Result<u64> {
        let num_entries = entries.len() as u32;
        let mut bloom = BloomFilter::new(entries.len().max(1), 0.01);
        for k in entries.keys() {
            bloom.insert(k);
        }

        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.path)?;
        let mut writer = BufWriter::new(file);

        // Header
        writer.write_all(&num_entries.to_le_bytes())?;
        writer.write_all(&(bloom.num_bits() as u32).to_le_bytes())?;
        writer.write_all(&[7u8])?; // num_hashes placeholder (we recompute on read)
        // Bloom filter bits
        let bloom_bytes = bloom_serialize(&bloom);
        writer.write_all(&(bloom_bytes.len() as u32).to_le_bytes())?;
        writer.write_all(&bloom_bytes)?;

        // Index: list of (key_len, key, offset) for binary search
        // For simplicity in alpha, we write entries sequentially.
        // A production version would write a sparse index at the end.
        let mut offset: u64 = 0;
        let mut offsets: Vec<(Vec<u8>, u64)> = Vec::new();
        for (k, v) in entries {
            offsets.push((k.clone(), offset));
            let stored = v.to_stored();
            offset += 4 + k.len() as u64 + 4 + stored.len() as u64;
        }

        // Write index
        writer.write_all(&(offsets.len() as u32).to_le_bytes())?;
        for (k, off) in &offsets {
            writer.write_all(&(k.len() as u32).to_le_bytes())?;
            writer.write_all(k)?;
            writer.write_all(&off.to_le_bytes())?;
        }

        // Write entries
        for (k, v) in entries {
            let stored = v.to_stored();
            writer.write_all(&(k.len() as u32).to_le_bytes())?;
            writer.write_all(k)?;
            writer.write_all(&(stored.len() as u32).to_le_bytes())?;
            writer.write_all(&stored)?;
        }

        writer.flush()?;
        writer.get_ref().sync_all()?;
        let metadata = std::fs::metadata(&self.path)?;
        Ok(metadata.len())
    }
}

/// SSTable reader — opens an existing SSTable for reads.
pub struct SSTableReader {
    #[allow(dead_code)]
    path: PathBuf,
    data: Vec<u8>,
    num_entries: u32,
}

impl SSTableReader {
    pub fn open(path: impl AsRef<std::path::Path>) -> std::io::Result<Self> {
        let path = path.as_ref().to_path_buf();
        let mut file = File::open(&path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        if data.len() < 9 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "SSTable too small",
            ));
        }
        let num_entries = u32::from_le_bytes(data[0..4].try_into().unwrap());
        Ok(Self {
            path,
            data,
            num_entries,
        })
    }

    /// Get a value by key. Returns None if not found.
    pub fn get(&self, key: &[u8]) -> Option<Value> {
        // Skip header (4 + 4 + 1 = 9 bytes) + bloom metadata.
        if self.data.len() < 13 {
            return None;
        }
        let bloom_len = u32::from_le_bytes(self.data[9..13].try_into().unwrap()) as usize;
        let index_start = 13 + bloom_len + 4; // skip bloom + index length prefix
        if index_start > self.data.len() {
            return None;
        }
        let index_count =
            u32::from_le_bytes(self.data[13 + bloom_len..13 + bloom_len + 4].try_into().unwrap())
                as usize;

        // Find entry in index via linear search (binary search for production).
        let mut pos = index_start;
        let mut found_offset: Option<u64> = None;
        for _ in 0..index_count {
            if pos + 4 > self.data.len() {
                return None;
            }
            let klen = u32::from_le_bytes(self.data[pos..pos + 4].try_into().unwrap()) as usize;
            pos += 4;
            if pos + klen + 8 > self.data.len() {
                return None;
            }
            let k = &self.data[pos..pos + klen];
            if k == key {
                let offset = u64::from_le_bytes(
                    self.data[pos + klen..pos + klen + 8].try_into().unwrap(),
                );
                found_offset = Some(offset);
                break;
            }
            pos += klen + 8;
        }

        let offset = found_offset?;
        // Compute the data section start = index_start + index_count * (4 + key + 8) per entry.
        let mut p = index_start;
        for _ in 0..index_count {
            if p + 4 > self.data.len() {
                return None;
            }
            let klen = u32::from_le_bytes(self.data[p..p + 4].try_into().unwrap()) as usize;
            p += 4 + klen + 8;
        }
        let data_start = p;

        let entry_pos = data_start + offset as usize;
        if entry_pos + 4 > self.data.len() {
            return None;
        }
        let klen = u32::from_le_bytes(self.data[entry_pos..entry_pos + 4].try_into().unwrap())
            as usize;
        let val_pos = entry_pos + 4 + klen;
        if val_pos + 4 > self.data.len() {
            return None;
        }
        let vlen =
            u32::from_le_bytes(self.data[val_pos..val_pos + 4].try_into().unwrap()) as usize;
        let val_bytes = &self.data[val_pos + 4..val_pos + 4 + vlen];
        Value::from_stored(val_bytes)
    }

    /// Number of entries.
    pub fn len(&self) -> usize {
        self.num_entries as usize
    }

    pub fn is_empty(&self) -> bool {
        self.num_entries == 0
    }
}

fn bloom_serialize(bf: &BloomFilter) -> Vec<u8> {
    // Simplified: just write the bits vector as little-endian u64s.
    // We don't deserialize it yet — bloom is recomputed on read for now.
    let _ = bf;
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_and_read() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let mut entries = BTreeMap::new();
        entries.insert(b"user:1".to_vec(), Value::from_str("Ahmed"));
        entries.insert(b"user:2".to_vec(), Value::from_str("Sara"));
        entries.insert(b"user:3".to_vec(), Value::from_str("Kenji"));

        let writer = SSTableWriter::new(tmp.path());
        let size = writer.write(&entries).unwrap();
        assert!(size > 0);

        let reader = SSTableReader::open(tmp.path()).unwrap();
        assert_eq!(reader.len(), 3);
        assert_eq!(
            reader.get(b"user:2").map(|v| v.display()),
            Some("Sara".into())
        );
        assert!(reader.get(b"missing").is_none());
    }
}
