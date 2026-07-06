# syntax=docker/dockerfile:1.7
# ====== Builder stage ======
FROM rust:1.83-alpine AS builder

WORKDIR /nawa

# Install build deps
RUN apk add --no-cache musl-dev

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
COPY crates/nawa-kernel/Cargo.toml ./crates/nawa-kernel/Cargo.toml
COPY crates/nawa-db/Cargo.toml ./crates/nawa-db/Cargo.toml
COPY crates/nawa-http/Cargo.toml ./crates/nawa-http/Cargo.toml
COPY crates/nawad/Cargo.toml ./crates/nawad/Cargo.toml

# Create dummy sources so cargo can resolve the workspace
RUN mkdir -p crates/nawa-kernel/src crates/nawa-db/src crates/nawa-http/src crates/nawad/src && \
    echo "pub fn _dummy() {}" > crates/nawa-kernel/src/lib.rs && \
    echo "pub fn _dummy() {}" > crates/nawa-db/src/lib.rs && \
    echo "pub fn _dummy() {}" > crates/nawa-http/src/lib.rs && \
    echo "fn main() {}" > crates/nawad/src/main.rs && \
    cargo build --release 2>/dev/null || true

# Now copy real sources and build
COPY crates/ ./crates/
RUN cargo build --release --bin nawad

# ====== Runtime stage (minimal) ======
FROM alpine:3.20

# musl + ca-certs only — no shell, no package manager needed
RUN apk add --no-cache ca-certificates tzdata && \
    adduser -D -u 10001 nawa

COPY --from=builder /nawa/target/release/nawad /usr/local/bin/nawad

USER nawa
EXPOSE 8080
HEALTHCHECK --interval=10s --timeout=2s \
    CMD wget --quiet --tries=1 --spider http://localhost:8080/health || exit 1

ENTRYPOINT ["/usr/local/bin/nawad"]
CMD ["serve", "--addr", "0.0.0.0:8080", "--data-dir", "/var/lib/nawa"]
