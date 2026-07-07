# nawa-cli — CLI Tool

The standalone CLI for creating, developing, and deploying NAWA projects.

## Commands

### nawa create

```bash
nawa create my-app --template saas
```

Creates a new project with:
- `Cargo.toml` with NAWA dependencies
- `src/main.rs` with 4 routes (/, /health, GET /:key, POST /:key)
- `.gitignore`, `README.md`, `Dockerfile`
- Directory structure: `src/routes/`, `src/db/`, `templates/`, `static/`

### nawa dev

```bash
nawa dev --addr 0.0.0.0:8080 --data-dir ./nawa-data
```

Starts the dev server with **hot reload**:
- Watches `src/` for changes (500ms polling)
- On change: kills server → rebuilds → restarts
- Build failures show error, keep old binary running

### nawa build

```bash
nawa build
```

Runs `cargo build --release` to produce an optimized binary.

### nawa deploy

```bash
nawa deploy --target ssh://user@your-vps --remote-data-dir /var/lib/nawa
```

Generates a deployment plan:
1. Build Docker image
2. Push to registry
3. SSH to target
4. Pull and run container

### nawa benchmark

```bash
nawa benchmark --ops 100000
```

Runs DB benchmarks via nawad.

### nawa info

```bash
nawa info
```

Shows: version, 6 components, platform (OS/arch/io_uring support).

### nawa templates

```bash
nawa templates
```

Lists 6 available templates:
- `blog` — Blog/CMS
- `saas` — Multi-tenant SaaS
- `shop` — E-commerce
- `realtime` — Chat app
- `booking` — Booking system
- `portfolio` — Personal portfolio
