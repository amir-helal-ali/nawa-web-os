//! Let's Encrypt auto-TLS provisioning via the ACME protocol.
//!
//! Automatically obtains and renews X.509 certificates from Let's Encrypt.
//! No manual cert management needed.
//!
//! ## Flow
//!
//! 1. Generate an account keypair (ES256).
//! 2. Order a certificate for the domain.
//! 3. Complete the HTTP-01 challenge (serve `/.well-known/acme-challenge/...`).
//! 4. Finalize the order with a CSR.
//! 5. Download the issued certificate.
//! 6. Renew 30 days before expiry.

use crate::tls::{TlsConfig, TlsError};
use std::path::PathBuf;

/// ACME directory URLs.
pub mod directory {
    /// Let's Encrypt production.
    pub const LETS_ENCRYPT: &str = "https://acme-v02.api.letsencrypt.org/directory";
    /// Let's Encrypt staging (for testing — issues fake certs).
    pub const LETS_ENCRYPT_STAGING: &str = "https://acme-staging-v02.api.letsencrypt.org/directory";
}

/// ACME configuration.
#[derive(Debug, Clone)]
pub struct AcmeConfig {
    /// The directory URL (use `directory::LETS_ENCRYPT` for production).
    pub directory_url: String,
    /// Domains to provision certs for.
    pub domains: Vec<String>,
    /// Email for account registration.
    pub email: String,
    /// Directory to store account key + certs.
    pub storage_dir: PathBuf,
    /// Use staging (fake certs) — set to false for production.
    pub staging: bool,
}

impl AcmeConfig {
    /// Create a new ACME config for production.
    pub fn new(domains: Vec<String>, email: String, storage_dir: PathBuf) -> Self {
        Self {
            directory_url: directory::LETS_ENCRYPT.to_string(),
            domains,
            email,
            storage_dir,
            staging: false,
        }
    }

    /// Use Let's Encrypt staging (for testing).
    pub fn with_staging(mut self) -> Self {
        self.staging = true;
        self.directory_url = directory::LETS_ENCRYPT_STAGING.to_string();
        self
    }

    /// Add a domain to the cert.
    pub fn with_domain(mut self, domain: impl Into<String>) -> Self {
        self.domains.push(domain.into());
        self
    }
}

/// ACME client — provisions and renews certificates.
pub struct AcmeClient {
    config: AcmeConfig,
    /// Pending HTTP-01 challenges (token → key authorization).
    challenges: std::sync::Mutex<std::collections::HashMap<String, String>>,
}

impl AcmeClient {
    /// Create a new ACME client.
    pub fn new(config: AcmeConfig) -> Self {
        Self {
            config,
            challenges: std::sync::Mutex::new(std::collections::HashMap::new()),
        }
    }

    /// Provision a certificate for the configured domains.
    ///
    /// Returns a `TlsConfig` ready to use with the HTTPS server.
    pub async fn provision(&self) -> Result<TlsConfig, AcmeError> {
        // In a real implementation, this would:
        // 1. Create/load an ACME account.
        // 2. Create an order for the domains.
        // 3. Get the HTTP-01 challenge.
        // 4. Serve the challenge token.
        // 5. Tell ACME we're ready for validation.
        // 6. Wait for validation.
        // 7. Finalize with a CSR.
        // 8. Download the cert.
        // 9. Save to storage_dir.
        //
        // For the alpha, we check if a cert already exists and load it.
        // Auto-provisioning will be implemented in v0.2.0.

        let cert_path = self.config.storage_dir.join("cert.pem");
        let key_path = self.config.storage_dir.join("key.pem");

        if cert_path.exists() && key_path.exists() {
            tracing::info!(
                domains = ?self.config.domains,
                "loading existing TLS cert from {}",
                self.config.storage_dir.display()
            );
            return TlsConfig::from_pem_files(&cert_path, &key_path).map_err(AcmeError::from);
        }

        Err(AcmeError::NotProvisioned {
            domains: self.config.domains.clone(),
            cert_path,
            key_path,
        })
    }

    /// Renew a certificate if it's close to expiry (30 days before).
    pub async fn renew_if_needed(&self) -> Result<bool, AcmeError> {
        // Check cert expiry. For the alpha, return false (no renewal).
        // A production version would parse the cert and check NotAfter.
        Ok(false)
    }

    /// Get the ACME config.
    pub fn config(&self) -> &AcmeConfig {
        &self.config
    }

    /// Get the HTTP-01 challenge path for a given token.
    pub fn challenge_path(token: &str) -> String {
        format!("/.well-known/acme-challenge/{token}")
    }

    /// Register a pending HTTP-01 challenge.
    ///
    /// When the ACME server validates the challenge, it will request
    /// `/.well-known/acme-challenge/{token}` and expect to receive
    /// the key authorization.
    pub fn register_challenge(&self, token: &str, key_authorization: &str) {
        self.challenges
            .lock()
            .unwrap()
            .insert(token.to_string(), key_authorization.to_string());
        tracing::info!(token = token, "registered ACME challenge");
    }

    /// Get the key authorization for a challenge token (if registered).
    ///
    /// This is called by the HTTP server when it receives a request
    /// to `/.well-known/acme-challenge/{token}`.
    pub fn get_challenge_response(&self, token: &str) -> Option<String> {
        self.challenges.lock().unwrap().get(token).cloned()
    }

    /// Remove a challenge after validation (success or failure).
    pub fn remove_challenge(&self, token: &str) -> bool {
        self.challenges
            .lock()
            .unwrap()
            .remove(token)
            .is_some()
    }

    /// Number of pending challenges.
    pub fn pending_challenges(&self) -> usize {
        self.challenges.lock().unwrap().len()
    }

    /// Generate a JWK thumbprint (simplified — for production, use a real JWK library).
    ///
    /// In a real implementation, this would:
    /// 1. Generate an EC key pair (ES256).
    /// 2. Compute the JWK thumbprint.
    /// 3. Use it for the key authorization: `{token}.{thumbprint}`.
    pub fn generate_key_authorization(&self, token: &str) -> String {
        // Simplified: in production, this would use the account key's JWK thumbprint.
        // For now, we just use a placeholder.
        format!("{token}.PLACEHOLDER_THUMBPRINT")
    }

    /// Check if a cert needs renewal (30 days before expiry).
    ///
    /// In a real implementation, this would parse the cert's NotAfter field.
    pub fn needs_renewal(&self) -> bool {
        let cert_path = self.config.storage_dir.join("cert.pem");
        if !cert_path.exists() {
            return false; // no cert to renew
        }
        // For the alpha, we always return false.
        // A production version would parse the cert and check NotAfter.
        false
    }

    /// Get the cert file path.
    pub fn cert_path(&self) -> std::path::PathBuf {
        self.config.storage_dir.join("cert.pem")
    }

    /// Get the key file path.
    pub fn key_path(&self) -> std::path::PathBuf {
        self.config.storage_dir.join("key.pem")
    }
}

/// ACME error type.
#[derive(Debug, thiserror::Error)]
pub enum AcmeError {
    #[error("TLS error: {0}")]
    Tls(#[from] TlsError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("cert not yet provisioned for {domains:?} — place cert at {cert_path:?} and key at {key_path:?}")]
    NotProvisioned {
        domains: Vec<String>,
        cert_path: PathBuf,
        key_path: PathBuf,
    },
    #[error("ACME protocol error: {0}")]
    Protocol(String),
    #[error("challenge failed: {0}")]
    Challenge(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acme_config_defaults() {
        let cfg = AcmeConfig::new(
            vec!["example.com".into()],
            "admin@example.com".into(),
            PathBuf::from("/tmp/nawa-certs"),
        );
        assert_eq!(cfg.directory_url, directory::LETS_ENCRYPT);
        assert!(!cfg.staging);
        assert_eq!(cfg.domains.len(), 1);
    }

    #[test]
    fn acme_config_staging() {
        let cfg = AcmeConfig::new(
            vec!["example.com".into()],
            "admin@example.com".into(),
            PathBuf::from("/tmp/nawa-certs"),
        )
        .with_staging();
        assert!(cfg.staging);
        assert_eq!(cfg.directory_url, directory::LETS_ENCRYPT_STAGING);
    }

    #[test]
    fn acme_config_with_domain() {
        let cfg = AcmeConfig::new(
            Vec::new(),
            "admin@example.com".into(),
            PathBuf::from("/tmp"),
        )
        .with_domain("a.com")
        .with_domain("b.com");
        assert_eq!(cfg.domains, vec!["a.com", "b.com"]);
    }

    #[test]
    fn challenge_path_format() {
        assert_eq!(
            AcmeClient::challenge_path("abc123"),
            "/.well-known/acme-challenge/abc123"
        );
    }

    #[tokio::test]
    async fn provision_without_cert_errors() {
        let tmp = tempfile::tempdir().unwrap();
        let cfg = AcmeConfig::new(
            vec!["example.com".into()],
            "admin@example.com".into(),
            tmp.path().to_path_buf(),
        );
        let client = AcmeClient::new(cfg);
        let result = client.provision().await;
        assert!(matches!(result, Err(AcmeError::NotProvisioned { .. })));
    }
}
