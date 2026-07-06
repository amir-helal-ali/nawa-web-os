//! TLS configuration for HTTPS support.
//!
//! Loads PEM-encoded certificate + private key and builds a `rustls::ServerConfig`.
//! In production, this would auto-provision Let's Encrypt certs; for now we accept
//! a manually-provided cert file.

use std::path::Path;
use std::sync::Arc;

/// A TLS configuration for the HTTPS server.
pub struct TlsConfig {
    inner: Arc<rustls::ServerConfig>,
}

impl TlsConfig {
    /// Load a TLS config from PEM cert + key files.
    pub fn from_pem_files<P: AsRef<Path>>(
        cert_path: P,
        key_path: P,
    ) -> Result<Self, TlsError> {
        let certs = load_certs(cert_path.as_ref())?;
        let key = load_private_key(key_path.as_ref())?;

        let cfg = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, key)
            .map_err(|e| TlsError::Config(e.to_string()))?;

        Ok(Self {
            inner: Arc::new(cfg),
        })
    }

    /// Get the underlying `ServerConfig`.
    pub fn config(&self) -> Arc<rustls::ServerConfig> {
        self.inner.clone()
    }
}

/// Generate a self-signed certificate for development.
///
/// In production, use Let's Encrypt via the `nawad` CLI.
pub fn self_signed() -> Result<TlsConfig, TlsError> {
    // This would use `rcgen` to generate a self-signed cert.
    // For now, return an error suggesting to provide a cert file.
    Err(TlsError::Config(
        "self-signed certs not yet implemented — provide a PEM cert file".into(),
    ))
}

/// TLS error type.
#[derive(Debug, thiserror::Error)]
pub enum TlsError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("cert parse error: {0}")]
    CertParse(String),
    #[error("key parse error: {0}")]
    KeyParse(String),
    #[error("config error: {0}")]
    Config(String),
}

fn load_certs(path: &Path) -> Result<Vec<rustls::pki_types::CertificateDer<'static>>, TlsError> {
    let file = std::fs::File::open(path)?;
    let mut reader = std::io::BufReader::new(file);
    let certs: Vec<_> = rustls_pemfile::certs(&mut reader)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| TlsError::CertParse(e.to_string()))?;
    if certs.is_empty() {
        return Err(TlsError::CertParse("no certificates found in file".into()));
    }
    Ok(certs.into_iter().map(|c| c.into_owned()).collect())
}

fn load_private_key(path: &Path) -> Result<rustls::pki_types::PrivateKeyDer<'static>, TlsError> {
    let file = std::fs::File::open(path)?;
    let mut reader = std::io::BufReader::new(file);
    let keys: Vec<_> = rustls_pemfile::private_key(&mut reader)
        .map(|opt| opt.into_iter().collect::<Vec<_>>())
        .unwrap_or_default();
    let key = keys.into_iter().next().ok_or_else(|| {
        TlsError::KeyParse("no private key found in file".into())
    })?;
    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_cert_file_errors() {
        let r = TlsConfig::from_pem_files("/nonexistent/cert.pem", "/nonexistent/key.pem");
        assert!(r.is_err());
    }
}
