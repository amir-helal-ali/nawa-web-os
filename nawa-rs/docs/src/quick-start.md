# Quick Start

## Install

### From Binary (Linux x86_64)

```bash
# Download the latest release
curl -L https://github.com/amir-helal-ali/nawa-web-os/releases/download/v0.1.0-alpha/nawa-v0.1.0-alpha-linux-amd64.tar.gz | tar xz

# Move to PATH
sudo mv nawad nawa /usr/local/bin/

# Verify
nawa info
```

### From Source

```bash
git clone https://github.com/amir-helal-ali/nawa-web-os.git
cd nawa-web-os/nawa-rs
cargo build --release
# Binaries in target/release/
```

## Create a Project

```bash
# List available templates
nawa templates

# Create a new project
nawa create my-app --template saas

# Enter the project
cd my-app
```

## Run the Dev Server

```bash
# Start with hot reload
nawa dev

# Or start nawad directly
nawad serve --addr 0.0.0.0:8080 --data-dir ./nawa-data
```

## Test the API

```bash
# Health check
curl http://localhost:8080/health

# Store a value
curl -X POST http://localhost:8080/user:1001 -d '{"name":"Ahmed","role":"admin"}'

# Retrieve it
curl http://localhost:8080/user:1001

# Scan with prefix
curl http://localhost:8080/scan/user:

# Prometheus metrics
curl http://localhost:8080/metrics
```

## Deploy

```bash
# Build production binary
nawa build

# Deploy to VPS
nawa deploy --target ssh://user@your-vps
```

## Available Templates

| Template | Description |
|----------|-------------|
| `blog` | Blog / CMS with SSR + SEO + admin panel |
| `saas` | Multi-tenant SaaS with subscriptions + billing |
| `shop` | E-commerce with cart + checkout + inventory |
| `realtime` | Realtime chat with WebSocket + presence |
| `booking` | Booking system with calendar + payments |
| `portfolio` | Portfolio with projects + contact form |
