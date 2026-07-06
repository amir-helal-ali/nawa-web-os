//! WASM plugin definition and manifest.

use serde::{Deserialize, Serialize};

/// A WASM plugin's manifest — declares its name, version, and capabilities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    /// Plugin name (e.g., "auth-jwt").
    pub name: String,
    /// Semantic version (e.g., "0.2.1").
    pub version: String,
    /// Author (e.g., "@nawa-official").
    pub author: String,
    /// Capabilities the plugin requests (e.g., "db:read", "http:fetch").
    pub capabilities: Vec<String>,
    /// Description (human-readable).
    pub description: String,
}

impl PluginManifest {
    /// Create a new manifest.
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            author: String::new(),
            capabilities: Vec::new(),
            description: String::new(),
        }
    }

    /// Set the author.
    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.author = author.into();
        self
    }

    /// Add a capability.
    pub fn with_capability(mut self, cap: impl Into<String>) -> Self {
        self.capabilities.push(cap.into());
        self
    }

    /// Set the description.
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Check if the plugin has a given capability.
    pub fn has_capability(&self, cap: &str) -> bool {
        self.capabilities.iter().any(|c| c == cap)
    }

    /// Parse a manifest from JSON.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        let v: serde_json::Value = serde_json::from_str(json)?;
        let name = v["name"].as_str().unwrap_or("unknown").to_string();
        let version = v["version"].as_str().unwrap_or("0.0.0").to_string();
        let author = v["author"].as_str().unwrap_or("").to_string();
        let description = v["description"].as_str().unwrap_or("").to_string();
        let capabilities = v["capabilities"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|c| c.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();
        Ok(Self {
            name,
            version,
            author,
            description,
            capabilities,
        })
    }

    /// Serialize to JSON.
    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

/// A loaded WASM plugin — its manifest + compiled bytecode.
#[derive(Debug)]
pub struct Plugin {
    pub manifest: PluginManifest,
    /// The raw WASM bytecode.
    pub bytecode: Vec<u8>,
    /// When the plugin was loaded (Unix timestamp).
    pub loaded_at: i64,
}

impl Plugin {
    /// Create a new plugin from bytecode and manifest.
    pub fn new(manifest: PluginManifest, bytecode: Vec<u8>) -> Self {
        Self {
            manifest,
            bytecode,
            loaded_at: chrono::Utc::now().timestamp(),
        }
    }

    /// Plugin name (convenience accessor).
    pub fn name(&self) -> &str {
        &self.manifest.name
    }

    /// Plugin version (convenience accessor).
    pub fn version(&self) -> &str {
        &self.manifest.version
    }

    /// Bytecode size in bytes.
    pub fn size(&self) -> usize {
        self.bytecode.len()
    }

    /// Check if the plugin has a given capability.
    pub fn has_capability(&self, cap: &str) -> bool {
        self.manifest.has_capability(cap)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_builder() {
        let m = PluginManifest::new("auth-jwt", "0.2.1")
            .with_author("@nawa-official")
            .with_description("JWT auth handler")
            .with_capability("db:read")
            .with_capability("http:fetch");

        assert_eq!(m.name, "auth-jwt");
        assert_eq!(m.version, "0.2.1");
        assert_eq!(m.author, "@nawa-official");
        assert!(m.has_capability("db:read"));
        assert!(m.has_capability("http:fetch"));
        assert!(!m.has_capability("fs:write"));
    }

    #[test]
    fn manifest_from_json() {
        let json = r#"{
            "name": "auth-jwt",
            "version": "0.2.1",
            "author": "@nawa-official",
            "description": "JWT auth handler",
            "capabilities": ["db:read", "http:fetch"]
        }"#;
        let m = PluginManifest::from_json(json).unwrap();
        assert_eq!(m.name, "auth-jwt");
        assert_eq!(m.version, "0.2.1");
        assert!(m.has_capability("db:read"));
        assert!(!m.has_capability("fs:write"));
    }

    #[test]
    fn plugin_creation() {
        let manifest = PluginManifest::new("test", "0.1.0");
        let plugin = Plugin::new(manifest, vec![0x00, 0x61, 0x73, 0x6d]); // WASM magic
        assert_eq!(plugin.name(), "test");
        assert_eq!(plugin.version(), "0.1.0");
        assert_eq!(plugin.size(), 4);
    }

    #[test]
    fn manifest_to_json_roundtrip() {
        let m = PluginManifest::new("p", "1.0.0")
            .with_capability("db:read");
        let json = m.to_json().unwrap();
        let m2 = PluginManifest::from_json(&json).unwrap();
        assert_eq!(m.name, m2.name);
        assert_eq!(m.version, m2.version);
        assert!(m2.has_capability("db:read"));
    }
}
