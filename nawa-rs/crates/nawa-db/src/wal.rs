//! Write-Ahead Log (WAL) — durability guarantee.
//!
//! Every write is appended to the WAL *before* it's applied to the MemTable.
//! On crash recovery, we replay the WAL to reconstruct the MemTable.

use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::Mutex;

use crate::Value;

/// A write-ahead log entry.
#[derive(Debug, Clone)]
pub struct WalEntry {
    pub seq: u64,
    pub op: WalOp,
    pub key: Vec<u8>,
    pub value: Option<Value>,
}

/// Type of WAL operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WalOp {
    Put,
    Delete,
}

impl WalOp {
    pub fn as_byte(&self) -> u8 {
        match self {
            WalOp::Put => 1,
            WalOp::Delete => 2,
        }
    }

    pub fn from_byte(b: u8) -> Option<Self> {
        match b {
            1 => Some(WalOp::Put),
            2 => Some(WalOp::Delete),
            _ => None,
        }
    }
}

/// The write-ahead log.
pub struct WriteAheadLog {
    file: Mutex<BufWriter<File>>,
    path: PathBuf,
    seq: std::sync::atomic::AtomicU64,
}

impl WriteAheadLog {
    /// Open (or create) a WAL at the given path.
    pub fn open(path: impl AsRef<std::path::Path>) -> std::io::Result<Self> {
        let path = path.as_ref().to_path_buf();
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .open(&path)?;
        // Start seq from the last entry in the file.
        let seq = Self::read_last_seq(&path).unwrap_or(0);
        Ok(Self {
            file: Mutex::new(BufWriter::new(file)),
            path,
            seq: std::sync::atomic::AtomicU64::new(seq),
        })
    }

    /// Append a PUT entry. Returns the assigned sequence number.
    pub fn append_put(&self, key: &[u8], value: &Value) -> std::io::Result<u64> {
        let seq = self.seq.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
        let entry = WalEntry {
            seq,
            op: WalOp::Put,
            key: key.to_vec(),
            value: Some(value.clone()),
        };
        self.write_entry(&entry)?;
        Ok(seq)
    }

    /// Append a DELETE entry.
    pub fn append_delete(&self, key: &[u8]) -> std::io::Result<u64> {
        let seq = self.seq.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
        let entry = WalEntry {
            seq,
            op: WalOp::Delete,
            key: key.to_vec(),
            value: None,
        };
        self.write_entry(&entry)?;
        Ok(seq)
    }

    /// Sync the WAL to disk (fsync).
    pub fn sync(&self) -> std::io::Result<()> {
        let mut guard = self.file.lock().unwrap();
        guard.flush()?;
        guard.get_ref().sync_all()
    }

    /// Replay all entries from the WAL (for crash recovery).
    pub fn replay(&self) -> std::io::Result<Vec<WalEntry>> {
        let mut entries = Vec::new();
        let bytes = std::fs::read(&self.path)?;
        let mut pos = 0;
        while pos < bytes.len() {
            if pos + 9 > bytes.len() {
                break; // truncated entry at the end
            }
            let seq = u64::from_le_bytes(bytes[pos..pos + 8].try_into().unwrap());
            let op_byte = bytes[pos + 8];
            let op = WalOp::from_byte(op_byte).ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, "bad WAL op byte")
            })?;
            pos += 9;

            if pos + 4 > bytes.len() {
                break;
            }
            let key_len = u32::from_le_bytes(bytes[pos..pos + 4].try_into().unwrap()) as usize;
            pos += 4;
            if pos + key_len > bytes.len() {
                break;
            }
            let key = bytes[pos..pos + key_len].to_vec();
            pos += key_len;

            let value = match op {
                WalOp::Put => {
                    if pos + 4 > bytes.len() {
                        break;
                    }
                    let val_len =
                        u32::from_le_bytes(bytes[pos..pos + 4].try_into().unwrap()) as usize;
                    pos += 4;
                    if pos + val_len > bytes.len() {
                        break;
                    }
                    let val_bytes = &bytes[pos..pos + val_len];
                    pos += val_len;
                    Value::from_stored(val_bytes)
                }
                WalOp::Delete => None,
            };

            entries.push(WalEntry {
                seq,
                op,
                key,
                value,
            });
        }
        Ok(entries)
    }

    /// Truncate the WAL (called after a successful MemTable flush).
    pub fn truncate(&self) -> std::io::Result<()> {
        drop(self.file.lock().unwrap());
        std::fs::write(&self.path, b"")?;
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .open(&self.path)?;
        *self.file.lock().unwrap() = BufWriter::new(file);
        self.seq.store(0, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }

    fn write_entry(&self, entry: &WalEntry) -> std::io::Result<()> {
        let mut guard = self.file.lock().unwrap();
        // Format: [seq:8][op:1][key_len:4][key][val_len:4][val] (val only for Put)
        guard.write_all(&entry.seq.to_le_bytes())?;
        guard.write_all(&[entry.op.as_byte()])?;
        guard.write_all(&(entry.key.len() as u32).to_le_bytes())?;
        guard.write_all(&entry.key)?;
        if let Some(v) = &entry.value {
            let stored = v.to_stored();
            guard.write_all(&(stored.len() as u32).to_le_bytes())?;
            guard.write_all(&stored)?;
        }
        guard.flush()?;
        guard.get_ref().sync_all()?;
        Ok(())
    }

    fn read_last_seq(path: &std::path::Path) -> std::io::Result<u64> {
        let wal = WriteAheadLog::open_internal_readonly(path)?;
        let entries = wal.replay()?;
        Ok(entries.last().map(|e| e.seq).unwrap_or(0))
    }

    fn open_internal_readonly(path: &std::path::Path) -> std::io::Result<Self> {
        let file = OpenOptions::new().read(true).open(path)?;
        Ok(Self {
            file: Mutex::new(BufWriter::new(file)),
            path: path.to_path_buf(),
            seq: std::sync::atomic::AtomicU64::new(0),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn append_and_replay() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let wal = WriteAheadLog::open(tmp.path()).unwrap();
        wal.append_put(b"user:1", &Value::from_str("Ahmed")).unwrap();
        wal.append_put(b"user:2", &Value::from_str("Sara")).unwrap();
        wal.append_delete(b"user:1").unwrap();
        wal.sync().unwrap();

        let entries = wal.replay().unwrap();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].op, WalOp::Put);
        assert_eq!(entries[0].key, b"user:1");
        assert_eq!(entries[2].op, WalOp::Delete);
    }

    #[test]
    fn truncate_clears_log() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let wal = WriteAheadLog::open(tmp.path()).unwrap();
        wal.append_put(b"k1", &Value::from_str("v1")).unwrap();
        wal.truncate().unwrap();
        let entries = wal.replay().unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn seq_increments() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let wal = WriteAheadLog::open(tmp.path()).unwrap();
        let s1 = wal.append_put(b"k1", &Value::from_str("v1")).unwrap();
        let s2 = wal.append_put(b"k2", &Value::from_str("v2")).unwrap();
        assert_eq!(s1, 1);
        assert_eq!(s2, 2);
    }
}
