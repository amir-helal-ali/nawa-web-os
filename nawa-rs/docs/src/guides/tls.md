# TLS / HTTPS

## Manual TLS

Provide PEM cert + key files:

```bash
nawad serve --addr 0.0.0.0:8443 --data-dir ./nawa-data --cert cert.pem --key key.pem
```

## Let's Encrypt (ACME)

NAWA includes an ACME client for automatic cert provisioning.

### Configuration

```rust
use nawa_http::{AcmeClient, AcmeConfig};

let config = AcmeConfig::new(
    vec!["example.com".into()],
    "admin@example.com".into(),
    "/var/lib/nawa/certs".into(),
);
let client = AcmeClient::new(config);

// Register HTTP-01 challenge
client.register_challenge("token", "key-auth");

// Serve challenge at: /.well-known/acme-challenge/{token}

// Provision cert
let tls = client.provision().await?;
```

### Staging vs Production

```rust
// Staging (fake certs — for testing)
let config = AcmeConfig::new(...).with_staging();

// Production (real certs)
let config = AcmeConfig::new(...); // default is production
```

## HTTP/3 (QUIC)

HTTP/3 requires TLS 1.3 (built into QUIC):

```rust
use nawa_http::{Http3Server, Http3Config, TlsConfig};

let tls = TlsConfig::from_pem_files("cert.pem", "key.pem")?;
let config = Http3Config::new("[::]:8443".parse()?, tls);
let server = Http3Server::new(config, router);
server.serve().await?; // serves HTTP/3 over UDP
```

## Cert Renewal

```rust
// Check if cert needs renewal (30 days before expiry)
if client.needs_renewal() {
    client.provision().await?; // re-provision
}
```
