//! Access Control List (ACL) — fine-grained permission system.
//!
//! Provides:
//! - Resource-based permissions (e.g., "users:read", "posts:write")
//! - Role inheritance (admin inherits all user permissions)
//! - Per-user permission grants/revocations
//! - Wildcard permissions (e.g., "users:*", "*:*")
//! - Permission checking with context

#![allow(dead_code)]

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

/// A permission string: "resource:action" (e.g., "users:read", "posts:write").
pub type Permission = String;

/// A role with associated permissions.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Role {
    pub name: String,
    pub permissions: Vec<Permission>,
    pub inherits: Vec<String>,
    pub description: String,
}

/// A user's ACL entry.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UserAcl {
    pub user_id: String,
    pub roles: Vec<String>,
    pub grants: Vec<Permission>,
    pub revocations: Vec<Permission>,
}

/// The ACL manager.
pub struct AclManager {
    roles: RwLock<HashMap<String, Role>>,
    user_acls: RwLock<HashMap<String, UserAcl>>,
}

impl AclManager {
    /// Create a new ACL manager with default roles.
    pub fn new() -> Arc<Self> {
        let mgr = Arc::new(Self {
            roles: RwLock::new(HashMap::new()),
            user_acls: RwLock::new(HashMap::new()),
        });
        // Default roles are set up synchronously via blocking_write alternative.
        // We'll use a simple approach: set up in new() via try_write.
        mgr
    }

    /// Initialize default roles.
    pub async fn init_defaults(&self) {
        // Admin role — full access.
        self.add_role(Role {
            name: "admin".into(),
            permissions: vec!["*:*".into()],
            inherits: vec!["user".into()],
            description: "Full system access".into(),
        }).await;

        // User role — basic access.
        self.add_role(Role {
            name: "user".into(),
            permissions: vec![
                "profile:read".into(),
                "profile:write".into(),
                "posts:read".into(),
                "posts:write".into(),
                "comments:read".into(),
                "comments:write".into(),
            ],
            inherits: vec![],
            description: "Standard user access".into(),
        }).await;

        // Guest role — read-only.
        self.add_role(Role {
            name: "guest".into(),
            permissions: vec![
                "posts:read".into(),
                "comments:read".into(),
            ],
            inherits: vec![],
            description: "Read-only guest access".into(),
        }).await;

        // API role — for API clients.
        self.add_role(Role {
            name: "api".into(),
            permissions: vec![
                "api:read".into(),
                "api:write".into(),
            ],
            inherits: vec!["guest".into()],
            description: "API client access".into(),
        }).await;
    }

    /// Add a role.
    pub async fn add_role(&self, role: Role) {
        self.roles.write().await.insert(role.name.clone(), role);
    }

    /// Assign a role to a user.
    pub async fn assign_role(&self, user_id: &str, role: &str) -> bool {
        let roles = self.roles.read().await;
        if !roles.contains_key(role) {
            return false;
        }
        drop(roles);

        let mut acls = self.user_acls.write().await;
        let acl = acls.entry(user_id.to_string()).or_insert(UserAcl {
            user_id: user_id.to_string(),
            roles: vec![],
            grants: vec![],
            revocations: vec![],
        });
        if !acl.roles.contains(&role.to_string()) {
            acl.roles.push(role.to_string());
        }
        true
    }

    /// Revoke a role from a user.
    pub async fn revoke_role(&self, user_id: &str, role: &str) -> bool {
        let mut acls = self.user_acls.write().await;
        if let Some(acl) = acls.get_mut(user_id) {
            let before = acl.roles.len();
            acl.roles.retain(|r| r != role);
            return acl.roles.len() < before;
        }
        false
    }

    /// Grant a specific permission to a user.
    pub async fn grant_permission(&self, user_id: &str, permission: &str) {
        let mut acls = self.user_acls.write().await;
        let acl = acls.entry(user_id.to_string()).or_insert(UserAcl {
            user_id: user_id.to_string(),
            roles: vec![],
            grants: vec![],
            revocations: vec![],
        });
        if !acl.grants.contains(&permission.to_string()) {
            acl.grants.push(permission.to_string());
        }
    }

    /// Revoke a specific permission from a user.
    pub async fn revoke_permission(&self, user_id: &str, permission: &str) {
        let mut acls = self.user_acls.write().await;
        let acl = acls.entry(user_id.to_string()).or_insert(UserAcl {
            user_id: user_id.to_string(),
            roles: vec![],
            grants: vec![],
            revocations: vec![],
        });
        if !acl.revocations.contains(&permission.to_string()) {
            acl.revocations.push(permission.to_string());
        }
    }

    /// Check if a user has a specific permission.
    pub async fn can(&self, user_id: &str, permission: &str) -> bool {
        let acls = self.user_acls.read().await;
        let roles = self.roles.read().await;

        let acl = match acls.get(user_id) {
            Some(a) => a,
            None => return false,
        };

        // 1. Check explicit revocations first (deny takes priority).
        if acl.revocations.iter().any(|p| permission_matches(p, permission)) {
            return false;
        }

        // 2. Check explicit grants.
        if acl.grants.iter().any(|p| permission_matches(p, permission)) {
            return true;
        }

        // 3. Check role-based permissions (with inheritance).
        for role_name in &acl.roles {
            if self.role_has_permission(&roles, role_name, permission, &mut HashSet::new()) {
                return true;
            }
        }

        false
    }

    /// Check if a user has any of the given permissions.
    pub async fn can_any(&self, user_id: &str, permissions: &[&str]) -> bool {
        for perm in permissions {
            if self.can(user_id, perm).await {
                return true;
            }
        }
        false
    }

    /// Check if a user has all of the given permissions.
    pub async fn can_all(&self, user_id: &str, permissions: &[&str]) -> bool {
        for perm in permissions {
            if !self.can(user_id, perm).await {
                return false;
            }
        }
        true
    }

    /// Get all permissions for a user (resolved with inheritance).
    pub async fn user_permissions(&self, user_id: &str) -> Vec<Permission> {
        let acls = self.user_acls.read().await;
        let roles = self.roles.read().await;
        let mut all_perms: HashSet<Permission> = HashSet::new();

        if let Some(acl) = acls.get(user_id) {
            // Add explicit grants.
            for g in &acl.grants {
                all_perms.insert(g.clone());
            }

            // Add role permissions.
            for role_name in &acl.roles {
                self.collect_role_permissions(&roles, role_name, &mut all_perms, &mut HashSet::new());
            }

            // Remove revocations.
            for r in &acl.revocations {
                all_perms.remove(r);
            }
        }

        all_perms.into_iter().collect()
    }

    /// Get user's ACL entry.
    pub async fn get_user_acl(&self, user_id: &str) -> Option<UserAcl> {
        self.user_acls.read().await.get(user_id).cloned()
    }

    /// List all roles.
    pub async fn list_roles(&self) -> Vec<Role> {
        self.roles.read().await.values().cloned().collect()
    }

    /// Recursively check if a role has a permission (with inheritance).
    fn role_has_permission(
        &self,
        roles: &HashMap<String, Role>,
        role_name: &str,
        permission: &str,
        visited: &mut HashSet<String>,
    ) -> bool {
        if visited.contains(role_name) {
            return false; // Prevent cycles.
        }
        visited.insert(role_name.to_string());

        let role = match roles.get(role_name) {
            Some(r) => r,
            None => return false,
        };

        // Check direct permissions.
        if role.permissions.iter().any(|p| permission_matches(p, permission)) {
            return true;
        }

        // Check inherited roles.
        for parent in &role.inherits {
            if self.role_has_permission(roles, parent, permission, visited) {
                return true;
            }
        }

        false
    }

    /// Recursively collect all permissions for a role.
    fn collect_role_permissions(
        &self,
        roles: &HashMap<String, Role>,
        role_name: &str,
        all_perms: &mut HashSet<Permission>,
        visited: &mut HashSet<String>,
    ) {
        if visited.contains(role_name) {
            return;
        }
        visited.insert(role_name.to_string());

        if let Some(role) = roles.get(role_name) {
            for p in &role.permissions {
                all_perms.insert(p.clone());
            }
            for parent in &role.inherits {
                self.collect_role_permissions(roles, parent, all_perms, visited);
            }
        }
    }
}

/// Check if a permission pattern matches a required permission.
/// Supports wildcards: "*" matches anything, "users:*" matches "users:read".
fn permission_matches(pattern: &str, required: &str) -> bool {
    if pattern == "*:*" {
        return true;
    }

    let pattern_parts: Vec<&str> = pattern.splitn(2, ':').collect();
    let required_parts: Vec<&str> = required.splitn(2, ':').collect();

    if pattern_parts.len() != 2 || required_parts.len() != 2 {
        return pattern == required;
    }

    let resource_match = pattern_parts[0] == "*" || pattern_parts[0] == required_parts[0];
    let action_match = pattern_parts[1] == "*" || pattern_parts[1] == required_parts[1];

    resource_match && action_match
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup() -> Arc<AclManager> {
        let mgr = AclManager::new();
        mgr.init_defaults().await;
        mgr
    }

    #[tokio::test]
    async fn admin_has_all_permissions() {
        let mgr = setup().await;
        mgr.assign_role("u1", "admin").await;
        assert!(mgr.can("u1", "users:read").await);
        assert!(mgr.can("u1", "posts:write").await);
        assert!(mgr.can("u1", "anything:anything").await);
    }

    #[tokio::test]
    async fn user_has_basic_permissions() {
        let mgr = setup().await;
        mgr.assign_role("u1", "user").await;
        assert!(mgr.can("u1", "profile:read").await);
        assert!(mgr.can("u1", "posts:write").await);
        assert!(!mgr.can("u1", "users:delete").await);
        assert!(!mgr.can("u1", "admin:settings").await);
    }

    #[tokio::test]
    async fn guest_has_read_only() {
        let mgr = setup().await;
        mgr.assign_role("u1", "guest").await;
        assert!(mgr.can("u1", "posts:read").await);
        assert!(!mgr.can("u1", "posts:write").await);
    }

    #[tokio::test]
    async fn explicit_grant_overrides_role() {
        let mgr = setup().await;
        mgr.assign_role("u1", "guest").await;
        assert!(!mgr.can("u1", "posts:write").await);
        mgr.grant_permission("u1", "posts:write").await;
        assert!(mgr.can("u1", "posts:write").await);
    }

    #[tokio::test]
    async fn revocation_overrides_everything() {
        let mgr = setup().await;
        mgr.assign_role("u1", "admin").await;
        assert!(mgr.can("u1", "posts:read").await);
        mgr.revoke_permission("u1", "posts:read").await;
        assert!(!mgr.can("u1", "posts:read").await);
    }

    #[tokio::test]
    async fn revoke_role_removes_permissions() {
        let mgr = setup().await;
        mgr.assign_role("u1", "admin").await;
        assert!(mgr.can("u1", "users:read").await);
        mgr.revoke_role("u1", "admin").await;
        assert!(!mgr.can("u1", "users:read").await);
    }

    #[tokio::test]
    async fn can_any_checks_multiple() {
        let mgr = setup().await;
        mgr.assign_role("u1", "user").await;
        assert!(mgr.can_any("u1", &["admin:settings", "posts:read"]).await);
        assert!(!mgr.can_any("u1", &["admin:settings", "users:delete"]).await);
    }

    #[tokio::test]
    async fn can_all_checks_all() {
        let mgr = setup().await;
        mgr.assign_role("u1", "user").await;
        assert!(mgr.can_all("u1", &["posts:read", "posts:write"]).await);
        assert!(!mgr.can_all("u1", &["posts:read", "users:delete"]).await);
    }

    #[tokio::test]
    async fn user_permissions_resolves_inheritance() {
        let mgr = setup().await;
        mgr.assign_role("u1", "admin").await;
        let perms = mgr.user_permissions("u1").await;
        assert!(perms.contains(&"*:*".to_string()));
    }

    #[tokio::test]
    async fn assign_nonexistent_role_fails() {
        let mgr = setup().await;
        assert!(!mgr.assign_role("u1", "nonexistent").await);
    }

    #[tokio::test]
    async fn no_roles_means_no_access() {
        let mgr = setup().await;
        assert!(!mgr.can("u1", "posts:read").await);
    }

    #[tokio::test]
    async fn list_roles_returns_all() {
        let mgr = setup().await;
        let roles = mgr.list_roles().await;
        assert!(roles.len() >= 4); // admin, user, guest, api
    }

    #[test]
    fn permission_matches_exact() {
        assert!(permission_matches("users:read", "users:read"));
        assert!(!permission_matches("users:read", "users:write"));
    }

    #[test]
    fn permission_matches_wildcard_action() {
        assert!(permission_matches("users:*", "users:read"));
        assert!(permission_matches("users:*", "users:write"));
        assert!(permission_matches("users:*", "users:delete"));
    }

    #[test]
    fn permission_matches_wildcard_all() {
        assert!(permission_matches("*:*", "users:read"));
        assert!(permission_matches("*:*", "posts:write"));
        assert!(permission_matches("*:*", "anything:anything"));
    }

    #[test]
    fn permission_matches_wildcard_resource() {
        assert!(permission_matches("*:read", "users:read"));
        assert!(permission_matches("*:read", "posts:read"));
        assert!(!permission_matches("*:read", "users:write"));
    }

    #[tokio::test]
    async fn api_role_inherits_guest() {
        let mgr = setup().await;
        mgr.assign_role("u1", "api").await;
        // API inherits guest, so should have posts:read.
        assert!(mgr.can("u1", "posts:read").await);
        // API has api:read.
        assert!(mgr.can("u1", "api:read").await);
        // API doesn't have posts:write.
        assert!(!mgr.can("u1", "posts:write").await);
    }
}
