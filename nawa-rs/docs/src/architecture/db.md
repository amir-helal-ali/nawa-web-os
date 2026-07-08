# nawa-db

Built-in hybrid Key-Value + Document database. No external DBMS required.

## Architecture

```
Write Path:
  PUT → WAL (fsync) → MemTable (skip-list)
                    → (when full) → SSTable L0
                                  → Compaction → L1...L7

Read Path:
  GET → MemTable → Bloom Filter → SSTable (newest first)
```

## Usage

```rust
use nawa_db::{DbEngine, DbConfig, Value};

let db = DbEngine::open_in_memory();

// Store a string
db.put("user:1001", Value::from_str("Ahmed"))?;

// Store a JSON document
db.put("user:1002", Value::from_json_str(r#"{"name":"Sara","age":28}"#)?)?;

// Retrieve
let v = db.get("user:1001").unwrap();
assert_eq!(v.display(), "Ahmed");

// Scan with prefix
let users = db.scan_prefix("user:", 100);

// Delete
db.delete("user:1001")?;
```

## Persistence

```rust
let config = DbConfig {
    data_dir: "/var/lib/nawa".into(),
    memtable_max_size: 4 * 1024 * 1024, // 4 MB
    wal_sync: true, // fsync on every write
};
let db = DbEngine::open(config)?;
```

## Async WAL Fsync (with io_uring)

Enable the `uring-wal` feature for async fsync via io_uring:

```toml
[dependencies]
nawa-db = { path = "...", features = ["uring-wal"] }
```

```rust
// With uring-wal feature:
db.wal().sync_async(&uring).await?;

// Without feature:
db.wal().sync_async().await?; // blocking fallback
```

## Modules

| Module | Description |
|--------|-------------|
| bloom | Bloom filter (xxh3, 1% FP rate) |
| memtable | In-memory sorted table (BTreeMap + RwLock) |
| skip_list | Lock-free skip-list with random levels |
| wal | Write-ahead log with fsync |
| sstable | Immutable on-disk sorted table |
| compaction | Merges SSTables to reclaim space |
| engine | Ties everything together + stats |

## Performance

```
PUT:     480,257 ops/sec
GET:   5,807,128 ops/sec
SCAN:  2,866,245 ops/sec
```

## Tests
- 28 unit tests
- 11 integration tests (including persistence across reopen)
