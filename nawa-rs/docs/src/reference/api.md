# API Endpoints

## System

### GET /
Returns system info and list of endpoints.

```json
{
  "name": "NAWA",
  "version": "0.1.0-alpha",
  "description": "Revolutionary Web Operating System built in Rust",
  "endpoints": [...]
}
```

### GET /health
Health check with DB stats.

```json
{
  "status": "ok",
  "keys": 42,
  "memtable_bytes": 1024,
  "stats": {
    "puts": 100,
    "gets": 500,
    "deletes": 10,
    "scans": 5,
    "flushes": 0
  }
}
```

### GET /uring
io_uring pipeline stats.

```json
{
  "real_uring": true,
  "sqpoll_enabled": false,
  "entries": 256,
  "stats": {
    "submitted": 1000,
    "completed": 998,
    "in_flight": 2,
    "bytes_transferred": 4096000,
    "errors": 0
  }
}
```

### GET /metrics
Prometheus metrics (text format).

## Database

### GET /:key
Retrieve a value.

**Response:** `200 OK` with body, or `404 Not Found`.

### POST /:key
Store a value. Body becomes the value.

**Response:** `200 OK` — `stored: {key}`

### DELETE /:key
Delete a value.

**Response:** `200 OK` — `deleted` or `404` — `key was not present`

### GET /scan/:prefix
Scan keys with a prefix.

```json
{
  "count": 3,
  "results": [
    {"key": "user:1", "value": "Ahmed"},
    {"key": "user:2", "value": "Sara"},
    {"key": "user:3", "value": "Kenji"}
  ]
}
```

## WASM Plugins

### GET /plugins
List loaded plugins.

```json
{
  "count": 2,
  "plugins": ["auth-jwt", "cache-redis"]
}
```

### POST /plugins/:name/invoke
Invoke a plugin function. Body = function name.

```json
{
  "plugin": "auth-jwt",
  "function": "verify",
  "result": 0,
  "status": "ok"
}
```
