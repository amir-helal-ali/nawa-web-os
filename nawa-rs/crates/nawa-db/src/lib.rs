//! # NAWA-DB
//!
//! A hybrid Key-Value + Document database built from scratch.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │            Write Path                   │
//! │                                         │
//! │   PUT ─→ WAL ─→ MemTable (skip-list)   │
//! │                    │                    │
//! │                    ▼ (when full)        │
//! │              SSTable L0                 │
//! │                    │                    │
//! │                    ▼ (compaction)       │
//! │              SSTable L1...L7            │
//! └─────────────────────────────────────────┘
//!
//! ┌─────────────────────────────────────────┐
//! │            Read Path                    │
//! │                                         │
//! │   GET ─→ MemTable ─→ Bloom ─→ SSTable  │
//! │              (newest first)             │
//! └─────────────────────────────────────────┘
//! ```

pub mod bloom;
pub mod engine;
pub mod memtable;
pub mod sstable;
pub mod wal;

pub use bloom::BloomFilter;
pub use engine::{DbConfig, DbEngine, DbError, DbResult, DbStats, StatsSnapshot};
pub use memtable::MemTable;
pub use sstable::SSTableWriter;
pub use wal::WriteAheadLog;

/// A database key. Any byte slice.
pub type Key = Vec<u8>;

/// A database value. Either raw bytes or a JSON document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    /// Raw bytes (KV mode).
    Bytes(Vec<u8>),
    /// JSON document (Document mode).
    Json(serde_json::Value),
}

impl Value {
    /// Create a bytes value from a string.
    pub fn from_str(s: &str) -> Self {
        Value::Bytes(s.as_bytes().to_vec())
    }

    /// Create a bytes value from an integer.
    pub fn from_i64(n: i64) -> Self {
        Value::Bytes(n.to_string().into_bytes())
    }

    /// Try to parse as JSON document.
    pub fn from_json_str(s: &str) -> Result<Self, serde_json::Error> {
        let v: serde_json::Value = serde_json::from_str(s)?;
        Ok(Value::Json(v))
    }

    /// Serialize to bytes for storage.
    pub fn to_stored(&self) -> Vec<u8> {
        match self {
            Value::Bytes(b) => {
                let mut out = vec![0u8]; // tag: 0 = bytes
                out.extend_from_slice(b);
                out
            }
            Value::Json(v) => {
                let mut out = vec![1u8]; // tag: 1 = json
                out.extend_from_slice(&serde_json::to_vec(v).unwrap_or_default());
                out
            }
        }
    }

    /// Deserialize from stored bytes.
    pub fn from_stored(b: &[u8]) -> Option<Self> {
        if b.is_empty() {
            return None;
        }
        match b[0] {
            0 => Some(Value::Bytes(b[1..].to_vec())),
            1 => serde_json::from_slice(&b[1..])
                .ok()
                .map(Value::Json),
            _ => None,
        }
    }

    /// Render as a string for display.
    pub fn display(&self) -> String {
        match self {
            Value::Bytes(b) => String::from_utf8_lossy(b).to_string(),
            Value::Json(v) => serde_json::to_string_pretty(v).unwrap_or_default(),
        }
    }
}
