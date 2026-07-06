# nawad — Server Binary

The main server binary that ties everything together.

## CLI

```bash
# Start HTTP server
nawad serve --addr 0.0.0.0:8080 --data-dir ./nawa-data

# Run benchmark
nawad benchmark --ops 100000

# Show info
nawad info
```

## API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/` | System info + endpoint list |
| GET | `/health` | Health check + DB stats |
| GET | `/uring` | io_uring pipeline stats |
| GET | `/metrics` | Prometheus metrics |
| GET | `/plugins` | WASM plugins list |
| GET | `/:key` | Get value from DB |
| POST | `/:key` | Store value in DB |
| DELETE | `/:key` | Delete value from DB |
| GET | `/scan/:prefix` | Scan keys with prefix |
| POST | `/plugins/:name/invoke` | Invoke WASM plugin |

## Prometheus Metrics

15 metrics exposed at `/metrics`:

```
nawa_db_puts_total
nawa_db_gets_total
nawa_db_deletes_total
nawa_db_scans_total
nawa_db_keys
nawa_db_memtable_bytes
nawa_db_flushes_total
nawa_uring_submitted_total
nawa_uring_completed_total
nawa_uring_in_flight
nawa_uring_errors_total
nawa_http_requests_total
nawa_http_errors_total
nawa_wasm_plugins_loaded
nawa_wasm_invocations_total
```

## Initialization Sequence

1. Parse CLI args (clap)
2. Initialize tracing/logger
3. Open NAWA-DB (with WAL replay)
4. Initialize WASM sandbox (wasmtime)
5. Initialize io_uring pipeline
6. Initialize Prometheus metrics
7. Register HTTP routes
8. Start HTTP server (accept loop)
