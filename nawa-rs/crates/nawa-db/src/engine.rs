//! The database engine — ties together MemTable, WAL, and SSTables.

use std::path::PathBuf;
use std::sync::Arc;

use crate::{memtable::MemTable, wal::WriteAheadLog, Value};

/// Database error type.
#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("key not found")]
    NotFound,
    #[error("database is read-only")]
    ReadOnly,
}

pub type DbResult<T> = std::result::Result<T, DbError>;

/// Database configuration.
#[derive(Debug, Clone)]
pub struct DbConfig {
    /// Directory where WAL and SSTables live.
    pub data_dir: PathBuf,
    /// MemTable flush threshold (bytes).
    pub memtable_max_size: usize,
    /// Whether to fsync the WAL on every write.
    pub wal_sync: bool,
}

impl Default for DbConfig {
    fn default() -> Self {
        Self {
            data_dir: std::env::temp_dir().join("nawa-db"),
            memtable_max_size: 4 * 1024 * 1024, // 4 MB
            wal_sync: true,
        }
    }
}

/// The main database engine.
pub struct DbEngine {
    config: DbConfig,
    memtable: Arc<MemTable>,
    wal: Arc<WriteAheadLog>,
    seq: std::sync::atomic::AtomicU64,
    stats: Arc<DbStats>,
}

/// Atomic statistics counters.
#[derive(Debug, Default)]
pub struct DbStats {
    pub puts: std::sync::atomic::AtomicU64,
    pub gets: std::sync::atomic::AtomicU64,
    pub deletes: std::sync::atomic::AtomicU64,
    pub scans: std::sync::atomic::AtomicU64,
    pub memtable_flushes: std::sync::atomic::AtomicU64,
    pub bytes_written: std::sync::atomic::AtomicU64,
    pub bytes_read: std::sync::atomic::AtomicU64,
}

impl DbStats {
    pub fn snapshot(&self) -> StatsSnapshot {
        StatsSnapshot {
            puts: self.puts.load(std::sync::atomic::Ordering::Relaxed),
            gets: self.gets.load(std::sync::atomic::Ordering::Relaxed),
            deletes: self.deletes.load(std::sync::atomic::Ordering::Relaxed),
            scans: self.scans.load(std::sync::atomic::Ordering::Relaxed),
            memtable_flushes: self.memtable_flushes.load(std::sync::atomic::Ordering::Relaxed),
            bytes_written: self.bytes_written.load(std::sync::atomic::Ordering::Relaxed),
            bytes_read: self.bytes_read.load(std::sync::atomic::Ordering::Relaxed),
        }
    }
}

#[derive(Debug, Clone)]
pub struct StatsSnapshot {
    pub puts: u64,
    pub gets: u64,
    pub deletes: u64,
    pub scans: u64,
    pub memtable_flushes: u64,
    pub bytes_written: u64,
    pub bytes_read: u64,
}

impl DbEngine {
    /// Open (or create) a database.
    pub fn open(config: DbConfig) -> DbResult<Self> {
        std::fs::create_dir_all(&config.data_dir)?;
        let wal_path = config.data_dir.join("wal.log");
        let wal = WriteAheadLog::open(&wal_path)?;
        let memtable = Arc::new(MemTable::new(config.memtable_max_size));

        // Replay WAL into MemTable.
        let entries = wal.replay()?;
        let last_seq = entries.last().map(|e| e.seq).unwrap_or(0);
        for entry in entries {
            match entry.op {
                crate::wal::WalOp::Put => {
                    if let Some(v) = entry.value {
                        memtable.put(entry.key, v);
                    }
                }
                crate::wal::WalOp::Delete => {
                    memtable.delete(&entry.key);
                }
            }
        }

        Ok(Self {
            config,
            memtable,
            wal: Arc::new(wal),
            seq: std::sync::atomic::AtomicU64::new(last_seq),
            stats: Arc::new(DbStats::default()),
        })
    }

    /// Open an in-memory database (no persistence).
    pub fn open_in_memory() -> Self {
        let config = DbConfig {
            data_dir: std::env::temp_dir().join(format!("nawa-db-{}", std::process::id())),
            ..Default::default()
        };
        // Use a temporary WAL file.
        std::fs::create_dir_all(&config.data_dir).ok();
        let wal = WriteAheadLog::open(config.data_dir.join("wal.log")).unwrap();
        Self {
            config,
            memtable: Arc::new(MemTable::new(4 * 1024 * 1024)),
            wal: Arc::new(wal),
            seq: std::sync::atomic::AtomicU64::new(0),
            stats: Arc::new(DbStats::default()),
        }
    }

    /// Insert a key-value pair.
    pub fn put<K: AsRef<[u8]>>(&self, key: K, value: Value) -> DbResult<u64> {
        let key = key.as_ref();
        let stored_size = key.len() + value.to_stored().len();
        let seq = self.wal.append_put(key, &value)?;
        if self.config.wal_sync {
            self.wal.sync()?;
        }
        self.memtable.put(key.to_vec(), value);
        self.stats.puts.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.stats
            .bytes_written
            .fetch_add(stored_size as u64, std::sync::atomic::Ordering::Relaxed);

        // Auto-flush if full.
        if self.memtable.is_full() {
            self.try_flush()?;
        }
        Ok(seq)
    }

    /// Get a value by key.
    pub fn get<K: AsRef<[u8]>>(&self, key: K) -> Option<Value> {
        self.stats.gets.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let v = self.memtable.get(key.as_ref());
        if let Some(v) = &v {
            self.stats
                .bytes_read
                .fetch_add(v.to_stored().len() as u64, std::sync::atomic::Ordering::Relaxed);
        }
        v
    }

    /// Delete a key.
    pub fn delete<K: AsRef<[u8]>>(&self, key: K) -> DbResult<bool> {
        let key = key.as_ref();
        let existed = self.memtable.get(key).is_some();
        self.wal.append_delete(key)?;
        if self.config.wal_sync {
            self.wal.sync()?;
        }
        self.memtable.delete(key);
        self.stats
            .deletes
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Ok(existed)
    }

    /// Scan keys with a prefix. Returns up to `limit` results.
    pub fn scan_prefix<K: AsRef<[u8]>>(&self, prefix: K, limit: usize) -> Vec<(Vec<u8>, Value)> {
        self.stats
            .scans
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.memtable.scan_prefix(prefix.as_ref(), limit)
    }

    /// Count of entries in the MemTable.
    pub fn len(&self) -> usize {
        self.memtable.len()
    }

    /// Is the database empty?
    pub fn is_empty(&self) -> bool {
        self.memtable.is_empty()
    }

    /// Current MemTable size in bytes.
    pub fn memtable_size(&self) -> usize {
        self.memtable.size()
    }

    /// Snapshot the current statistics.
    pub fn stats(&self) -> StatsSnapshot {
        self.stats.snapshot()
    }

    /// Try to flush the MemTable to an SSTable. Returns the number of entries flushed.
    fn try_flush(&self) -> DbResult<usize> {
        let drained = self.memtable.drain();
        if drained.is_empty() {
            return Ok(0);
        }
        let count = drained.len();
        let sstable_path = self
            .config
            .data_dir
            .join(format!("sstable-{}.sst", chrono::Utc::now().timestamp_micros()));
        let writer = crate::sstable::SSTableWriter::new(&sstable_path);
        writer.write(&drained)?;
        self.wal.truncate()?;
        self.stats
            .memtable_flushes
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn put_get_delete() {
        let db = DbEngine::open_in_memory();
        db.put("user:1", Value::from_str("Ahmed")).unwrap();
        db.put("user:2", Value::from_str("Sara")).unwrap();

        assert_eq!(
            db.get("user:1").map(|v| v.display()),
            Some("Ahmed".into())
        );
        assert_eq!(db.len(), 2);

        db.delete("user:1").unwrap();
        assert!(db.get("user:1").is_none());
        assert_eq!(db.len(), 1);
    }

    #[test]
    fn json_documents() {
        let db = DbEngine::open_in_memory();
        let doc = Value::from_json_str(r#"{"name":"Ahmed","age":30}"#).unwrap();
        db.put("user:1", doc).unwrap();

        let v = db.get("user:1").unwrap();
        assert!(matches!(v, Value::Json(_)));
        assert!(v.display().contains("Ahmed"));
    }

    #[test]
    fn scan_prefix() {
        let db = DbEngine::open_in_memory();
        db.put("user:1", Value::from_str("A")).unwrap();
        db.put("user:2", Value::from_str("B")).unwrap();
        db.put("post:1", Value::from_str("P1")).unwrap();
        db.put("user:3", Value::from_str("C")).unwrap();

        let users = db.scan_prefix("user:", 100);
        assert_eq!(users.len(), 3);
    }

    #[test]
    fn stats_track() {
        let db = DbEngine::open_in_memory();
        db.put("k1", Value::from_str("v1")).unwrap();
        db.put("k2", Value::from_str("v2")).unwrap();
        db.get("k1");
        db.delete("k1").unwrap();
        db.scan_prefix("k", 10);

        let stats = db.stats();
        assert_eq!(stats.puts, 2);
        assert_eq!(stats.gets, 1);
        assert_eq!(stats.deletes, 1);
        assert_eq!(stats.scans, 1);
    }

    #[test]
    fn persistence_across_reopen() {
        let tmp = tempfile::tempdir().unwrap();
        let config = DbConfig {
            data_dir: tmp.path().to_path_buf(),
            memtable_max_size: 4 * 1024 * 1024,
            wal_sync: true,
        };
        {
            let db = DbEngine::open(config.clone()).unwrap();
            db.put("user:1", Value::from_str("Ahmed")).unwrap();
            db.put("user:2", Value::from_str("Sara")).unwrap();
        }
        // Reopen.
        let db = DbEngine::open(config).unwrap();
        assert_eq!(
            db.get("user:1").map(|v| v.display()),
            Some("Ahmed".into())
        );
        assert_eq!(
            db.get("user:2").map(|v| v.display()),
            Some("Sara".into())
        );
    }
}
