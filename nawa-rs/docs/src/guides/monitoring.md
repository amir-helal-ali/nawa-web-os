# Monitoring

## Prometheus Metrics

NAWA exposes 15 metrics at `/metrics` in Prometheus format:

```bash
curl http://localhost:8080/metrics
```

### Database Metrics

| Metric | Type | Description |
|--------|------|-------------|
| `nawa_db_puts_total` | counter | Total PUT operations |
| `nawa_db_gets_total` | counter | Total GET operations |
| `nawa_db_deletes_total` | counter | Total DELETE operations |
| `nawa_db_scans_total` | counter | Total SCAN operations |
| `nawa_db_keys` | gauge | Current number of keys |
| `nawa_db_memtable_bytes` | gauge | Current MemTable size |
| `nawa_db_flushes_total` | counter | Total MemTable flushes |

### io_uring Metrics

| Metric | Type | Description |
|--------|------|-------------|
| `nawa_uring_submitted_total` | counter | Total submissions |
| `nawa_uring_completed_total` | counter | Total completions |
| `nawa_uring_in_flight` | gauge | Current in-flight ops |
| `nawa_uring_errors_total` | counter | Total errors |

### HTTP Metrics

| Metric | Type | Description |
|--------|------|-------------|
| `nawa_http_requests_total` | counter | Total HTTP requests |
| `nawa_http_errors_total` | counter | Total 5xx errors |

### WASM Metrics

| Metric | Type | Description |
|--------|------|-------------|
| `nawa_wasm_plugins_loaded` | gauge | Loaded plugins |
| `nawa_wasm_invocations_total` | counter | Plugin invocations |

## Grafana Dashboard

Import this Prometheus datasource and query:

```promql
# Requests per second
rate(nawa_http_requests_total[1m])

# DB ops per second
rate(nawa_db_puts_total[1m]) + rate(nawa_db_gets_total[1m])

# io_uring throughput
rate(nawa_uring_completed_total[1m])

# Error rate
rate(nawa_http_errors_total[1m]) / rate(nawa_http_requests_total[1m])
```

## Health Check

```bash
curl http://localhost:8080/health
# → {"keys":42,"memtable_bytes":1024,"stats":{"puts":100,"gets":500,...},"status":"ok"}
```

## io_uring Stats

```bash
curl http://localhost:8080/uring
# → {"real_uring":true,"sqpoll_enabled":false,"entries":256,"stats":{...}}
```
