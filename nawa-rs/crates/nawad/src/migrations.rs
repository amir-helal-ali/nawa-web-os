//! Database migration system — versioned schema changes.
//!
//! Provides:
//! - Migration versioning (track applied migrations)
//! - Forward-only migrations (no rollback in production)
//! - Migration history (stored in NAWA-DB)
//! - Automatic migration on startup

#![allow(dead_code)]

use std::sync::Arc;

/// A single migration.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Migration {
    pub version: u32,
    pub name: String,
    pub description: String,
    pub applied_at: Option<String>,
}

/// Migration manager.
pub struct MigrationManager {
    db: Arc<nawa_db::DbEngine>,
    migrations: Vec<Migration>,
}

impl MigrationManager {
    /// Create a new migration manager.
    pub fn new(db: Arc<nawa_db::DbEngine>) -> Self {
        Self {
            db,
            migrations: Self::built_in_migrations(),
        }
    }

    /// Get the current schema version.
    pub fn current_version(&self) -> u32 {
        self.db.get("schema:version")
            .map(|v| v.display().parse::<u32>().unwrap_or(0))
            .unwrap_or(0)
    }

    /// Check if a migration has been applied.
    pub fn is_applied(&self, version: u32) -> bool {
        self.db.get(format!("migration:{version}")).is_some()
    }

    /// Run all pending migrations.
    pub fn run_pending(&self) -> Vec<Migration> {
        let current = self.current_version();
        let mut applied = Vec::new();

        for migration in &self.migrations {
            if migration.version <= current {
                continue;
            }

            // Apply migration.
            if let Err(e) = self.apply_migration(migration) {
                tracing::error!("Migration {} failed: {e}", migration.version);
                break;
            }

            applied.push(migration.clone());
            tracing::info!("✓ Migration {}: {}", migration.version, migration.name);
        }

        // Update schema version.
        if let Some(last) = self.migrations.last() {
            let _ = self.db.put("schema:version", nawa_db::Value::from_i64(last.version as i64));
        }

        applied
    }

    /// Apply a single migration.
    fn apply_migration(&self, migration: &Migration) -> Result<(), String> {
        // Mark as applied.
        let record = serde_json::json!({
            "version": migration.version,
            "name": migration.name,
            "description": migration.description,
            "applied_at": chrono::Utc::now().to_rfc3339()
        });
        let key = format!("migration:{}", migration.version);
        let _ = self.db.put(&key, nawa_db::Value::from_json_str(&record.to_string())
            .unwrap_or_else(|_| nawa_db::Value::from_str(&record.to_string())));

        // Execute migration-specific data setup.
        match migration.version {
            1 => self.migration_001_initial(),
            2 => self.migration_002_auth(),
            3 => self.migration_003_seo(),
            4 => self.migration_004_quantum(),
            5 => self.migration_005_plugins(),
            _ => Ok(()),
        }
    }

    /// Get migration history.
    pub fn history(&self) -> Vec<Migration> {
        let mut history = Vec::new();
        for migration in &self.migrations {
            let applied = self.is_applied(migration.version);
            let mut m = migration.clone();
            if applied {
                if let Some(val) = self.db.get(format!("migration:{}", migration.version)) {
                    if let Ok(data) = serde_json::from_str::<serde_json::Value>(&val.display()) {
                        m.applied_at = data["applied_at"].as_str().map(|s| s.to_string());
                    }
                }
            }
            history.push(m);
        }
        history
    }

    /// Get pending migrations.
    pub fn pending(&self) -> Vec<Migration> {
        let current = self.current_version();
        self.migrations.iter()
            .filter(|m| m.version > current)
            .cloned()
            .collect()
    }

    /// Get migration statistics.
    pub fn stats(&self) -> MigrationStats {
        let total = self.migrations.len();
        let applied = self.migrations.iter().filter(|m| self.is_applied(m.version)).count();
        MigrationStats {
            current_version: self.current_version(),
            total_migrations: total,
            applied_migrations: applied,
            pending_migrations: total - applied,
        }
    }

    /// Built-in migrations.
    fn built_in_migrations() -> Vec<Migration> {
        vec![
            Migration {
                version: 1,
                name: "initial".into(),
                description: "Initial schema — creates system keys".into(),
                applied_at: None,
            },
            Migration {
                version: 2,
                name: "auth".into(),
                description: "Auth system — user storage, settings, sessions".into(),
                applied_at: None,
            },
            Migration {
                version: 3,
                name: "seo".into(),
                description: "AION SEO — snapshots, canonicals, sitemap cache".into(),
                applied_at: None,
            },
            Migration {
                version: 4,
                name: "quantum".into(),
                description: "Quantum engine — tunneling state, entanglement registry".into(),
                applied_at: None,
            },
            Migration {
                version: 5,
                name: "plugins".into(),
                description: "Plugin system — plugin registry, hook cache".into(),
                applied_at: None,
            },
        ]
    }

    // ── Individual migrations ──

    fn migration_001_initial(&self) -> Result<(), String> {
        let _ = self.db.put("system:created_at", nawa_db::Value::from_str(&chrono::Utc::now().to_rfc3339()));
        let _ = self.db.put("system:version", nawa_db::Value::from_str("2.1.0"));
        Ok(())
    }

    fn migration_002_auth(&self) -> Result<(), String> {
        let _ = self.db.put("auth:settings", nawa_db::Value::from_json_str(
            r#"{"registration_open":true,"verification_required":false,"max_users":null}"#
        ).unwrap_or_else(|_| nawa_db::Value::from_str("{}")));
        Ok(())
    }

    fn migration_003_seo(&self) -> Result<(), String> {
        let _ = self.db.put("seo:config", nawa_db::Value::from_json_str(
            r#"{"sitemap_enabled":true,"robots_enabled":true,"og_enabled":true}"#
        ).unwrap_or_else(|_| nawa_db::Value::from_str("{}")));
        Ok(())
    }

    fn migration_004_quantum(&self) -> Result<(), String> {
        let _ = self.db.put("quantum:config", nawa_db::Value::from_json_str(
            r#"{"tunneling_enabled":true,"default_temperature":50.0,"cooling_rate":0.001}"#
        ).unwrap_or_else(|_| nawa_db::Value::from_str("{}")));
        Ok(())
    }

    fn migration_005_plugins(&self) -> Result<(), String> {
        let _ = self.db.put("plugins:config", nawa_db::Value::from_json_str(
            r#"{"auto_load":true,"sandbox_enabled":true,"fuel_limit":1000000}"#
        ).unwrap_or_else(|_| nawa_db::Value::from_str("{}")));
        Ok(())
    }
}

/// Migration statistics.
#[derive(Debug, Clone, serde::Serialize)]
pub struct MigrationStats {
    pub current_version: u32,
    pub total_migrations: usize,
    pub applied_migrations: usize,
    pub pending_migrations: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_mgr() -> MigrationManager {
        MigrationManager::new(Arc::new(nawa_db::DbEngine::open_in_memory()))
    }

    #[test]
    fn current_version_starts_at_zero() {
        let mgr = make_mgr();
        assert_eq!(mgr.current_version(), 0);
    }

    #[test]
    fn run_pending_applies_all() {
        let mgr = make_mgr();
        let applied = mgr.run_pending();
        assert_eq!(applied.len(), 5);
        assert_eq!(mgr.current_version(), 5);
    }

    #[test]
    fn run_pending_after_applied_does_nothing() {
        let mgr = make_mgr();
        mgr.run_pending();
        let applied = mgr.run_pending();
        assert!(applied.is_empty());
    }

    #[test]
    fn is_applied_checks_correctly() {
        let mgr = make_mgr();
        assert!(!mgr.is_applied(1));
        mgr.run_pending();
        assert!(mgr.is_applied(1));
        assert!(mgr.is_applied(5));
    }

    #[test]
    fn history_shows_all() {
        let mgr = make_mgr();
        mgr.run_pending();
        let history = mgr.history();
        assert_eq!(history.len(), 5);
        assert!(history[0].applied_at.is_some());
    }

    #[test]
    fn pending_shows_unapplied() {
        let mgr = make_mgr();
        assert_eq!(mgr.pending().len(), 5);
        mgr.run_pending();
        assert_eq!(mgr.pending().len(), 0);
    }

    #[test]
    fn stats_track_correctly() {
        let mgr = make_mgr();
        let stats = mgr.stats();
        assert_eq!(stats.current_version, 0);
        assert_eq!(stats.pending_migrations, 5);
        mgr.run_pending();
        let stats = mgr.stats();
        assert_eq!(stats.current_version, 5);
        assert_eq!(stats.applied_migrations, 5);
        assert_eq!(stats.pending_migrations, 0);
    }

    #[test]
    fn migration_creates_system_keys() {
        let mgr = make_mgr();
        mgr.run_pending();
        assert!(mgr.db.get("system:created_at").is_some());
        assert!(mgr.db.get("system:version").is_some());
    }

    #[test]
    fn migration_creates_auth_settings() {
        let mgr = make_mgr();
        mgr.run_pending();
        assert!(mgr.db.get("auth:settings").is_some());
    }

    #[test]
    fn migration_creates_seo_config() {
        let mgr = make_mgr();
        mgr.run_pending();
        assert!(mgr.db.get("seo:config").is_some());
    }

    #[test]
    fn migration_creates_quantum_config() {
        let mgr = make_mgr();
        mgr.run_pending();
        assert!(mgr.db.get("quantum:config").is_some());
    }

    #[test]
    fn migration_creates_plugins_config() {
        let mgr = make_mgr();
        mgr.run_pending();
        assert!(mgr.db.get("plugins:config").is_some());
    }
}
