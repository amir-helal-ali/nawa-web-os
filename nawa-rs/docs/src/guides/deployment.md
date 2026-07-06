# Deployment

## nawa deploy — SSH Deployment

```bash
nawa deploy --target user@your-vps --remote-data-dir /var/lib/nawa
```

This runs a 4-step deployment:

1. **Build**: `cargo build --release -p nawad`
2. **Package**: Creates tarball of the binary
3. **Upload**: SCP to remote `/tmp/`
4. **Install + Start**: SSH commands to:
   - Extract binary
   - Move to `/usr/local/bin/`
   - Kill old process
   - Start new server with `nohup`

## Docker Deployment

```bash
# Build image
docker build -t nawa-app .

# Run on 512MB VPS
docker run -d \
  --name nawa-prod \
  -p 80:8080 \
  -v nawa-data:/var/lib/nawa \
  --memory=512m \
  --cpus=1.0 \
  nawa-app
```

## docker-compose

```yaml
services:
  nawa:
    image: nawa/os:0.1.0
    ports:
      - "80:8080"
      - "443:8443/udp"  # HTTP/3 QUIC
    volumes:
      - nawa-data:/var/lib/nawa
    deploy:
      resources:
        limits:
          cpus: "1.0"
          memory: 512M
```

```bash
docker compose up -d
```

## Post-Deploy Verification

```bash
# Health check
curl http://your-vps:8080/health

# Metrics
curl http://your-vps:8080/metrics

# io_uring stats
curl http://your-vps:8080/uring
```
