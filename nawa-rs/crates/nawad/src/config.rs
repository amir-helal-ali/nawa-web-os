//! Configuration file support — nawa.toml.
//!
//! NAWA reads a TOML config file for server settings.
//! If no config file exists, defaults are used.

use std::path::Path;

/// Server configuration (from nawa.toml or defaults).
#[derive(Debug, Clone)]
pub struct Config {
    /// Bind address.
    pub addr: String,
    /// Data directory for NAWA-DB.
    pub data_dir: String,
    /// Plugins directory for WASM.
    pub plugins_dir: String,
    /// Static files directory.
    pub static_dir: String,
    /// WAL sync (true = fsync on every write).
    pub wal_sync: bool,
    /// JWT secret key.
    pub jwt_secret: String,
    /// Max requests per minute (rate limiting).
    pub rate_limit: u32,
    /// Log level (error, warn, info, debug, trace).
    pub log_level: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            addr: "0.0.0.0:8080".into(),
            data_dir: "./nawa-data".into(),
            plugins_dir: "./plugins".into(),
            static_dir: "./static".into(),
            wal_sync: true,
            jwt_secret: "nawa-os-secret-2026".into(),
            rate_limit: 100,
            log_level: "info".into(),
        }
    }
}

impl Config {
    /// Load config from a nawa.toml file, or return defaults if not found.
    pub fn load(path: &Path) -> Self {
        if !path.exists() {
            return Self::default();
        }

        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return Self::default(),
        };

        Self::parse_toml(&content)
    }

    /// Parse a TOML-like config string (simplified parser — no external toml crate).
    fn parse_toml(content: &str) -> Self {
        let mut config = Self::default();

        for line in content.lines() {
            let line = line.trim();

            // Skip comments and empty lines.
            if line.is_empty() || line.starts_with('#') || line.starts_with('[') {
                continue;
            }

            // Parse key = value.
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim().trim_matches('"');

                match key {
                    "addr" => config.addr = value.into(),
                    "data_dir" => config.data_dir = value.into(),
                    "plugins_dir" => config.plugins_dir = value.into(),
                    "static_dir" => config.static_dir = value.into(),
                    "wal_sync" => config.wal_sync = value == "true",
                    "jwt_secret" => config.jwt_secret = value.into(),
                    "rate_limit" => config.rate_limit = value.parse().unwrap_or(100),
                    "log_level" => config.log_level = value.into(),
                    _ => {}
                }
            }
        }

        config
    }

    /// Generate a default nawa.toml file.
    pub fn generate_default(path: &Path) -> std::io::Result<()> {
        let content = r#"# NAWA Configuration File
# https://github.com/amir-helal-ali/nawa-web-os

# Server bind address
addr = "0.0.0.0:8080"

# Data directory for NAWA-DB (LSM tree + WAL + SSTables)
data_dir = "./nawa-data"

# Plugins directory for WASM auto-loading
plugins_dir = "./plugins"

# Static files directory
static_dir = "./static"

# WAL sync (true = fsync every write, false = faster but less durable)
wal_sync = true

# JWT secret key (CHANGE THIS in production!)
jwt_secret = "nawa-os-secret-2026"

# Rate limit: max requests per minute per IP
rate_limit = 100

# Log level: error, warn, info, debug, trace
log_level = "info"
"#;
        std::fs::write(path, content)
    }

    /// Print config summary.
    pub fn summary(&self) -> String {
        format!(
            "addr={}, data_dir={}, plugins_dir={}, wal_sync={}, rate_limit={}, log_level={}",
            self.addr, self.data_dir, self.plugins_dir, self.wal_sync, self.rate_limit, self.log_level
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let c = Config::default();
        assert_eq!(c.addr, "0.0.0.0:8080");
        assert!(c.wal_sync);
        assert_eq!(c.rate_limit, 100);
    }

    #[test]
    fn parse_toml() {
        let toml = r#"
# comment
addr = "127.0.0.1:3000"
data_dir = "/var/lib/nawa"
wal_sync = false
rate_limit = 200
log_level = "debug"
"#;
        let c = Config::parse_toml(toml);
        assert_eq!(c.addr, "127.0.0.1:3000");
        assert_eq!(c.data_dir, "/var/lib/nawa");
        assert!(!c.wal_sync);
        assert_eq!(c.rate_limit, 200);
        assert_eq!(c.log_level, "debug");
    }

    #[test]
    fn load_missing_file_returns_default() {
        let c = Config::load(Path::new("/nonexistent/nawa.toml"));
        assert_eq!(c.addr, "0.0.0.0:8080");
    }

    #[test]
    fn generate_default_creates_file() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("nawa.toml");
        Config::generate_default(&path).unwrap();
        assert!(path.exists());

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("addr ="));
        assert!(content.contains("jwt_secret ="));
    }

    #[test]
    fn summary_works() {
        let c = Config::default();
        let s = c.summary();
        assert!(s.contains("addr="));
        assert!(s.contains("wal_sync="));
    }
}
