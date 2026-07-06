# Using the Database

## Basic Operations

```bash
# Store a value
curl -X POST http://localhost:8080/user:1001 -d '{"name":"Ahmed","role":"admin"}'
# → stored: user:1001

# Retrieve
curl http://localhost:8080/user:1001
# → {"name":"Ahmed","role":"admin"}

# Scan with prefix
curl http://localhost:8080/scan/user:
# → {"count":1,"results":[{"key":"user:1001","value":"..."}]}

# Delete
curl -X DELETE http://localhost:8080/user:1001
# → deleted

# 404 for missing keys
curl http://localhost:8080/user:1001
# → key not found: user:1001 (HTTP 404)
```

## Value Types

NAWA-DB supports two value types:

### KV (raw bytes)
```bash
curl -X POST http://localhost:8080/counter:visits -d '14823'
```

### Document (JSON)
```bash
curl -X POST http://localhost:8080/user:1001 -d '{"name":"Ahmed","age":30}'
```

The server auto-detects JSON (starts with `{` or `[`).

## Key Naming Conventions

Use `:` as namespace separator:

```
user:1001          → user document
user:1001:posts    → user's posts
session:abc123     → session token
config:smtp        → app config
counter:visits     → counter
post:42            → blog post
tag:rust           → tag index
```

## Performance

```
PUT:     480,257 ops/sec
GET:   5,807,128 ops/sec
SCAN:  2,866,245 ops/sec
```

## Persistence

Data is persisted to the `--data-dir` directory:
- `wal.log` — write-ahead log (replayed on restart)
- `sstable-*.sst` — immutable sorted tables
