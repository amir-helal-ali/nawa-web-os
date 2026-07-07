//! Role-based access control (RBAC).

/// User role.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Role {
    /// Full access — can manage users, change settings, view dashboard.
    Admin,
    /// Standard user — can use the app, view own profile.
    User,
}

impl Role {
    pub fn as_str(&self) -> &'static str {
        match self {
            Role::Admin => "admin",
            Role::User => "user",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "admin" => Some(Role::Admin),
            "user" => Some(Role::User),
            _ => None,
        }
    }

    /// Is this role an admin?
    pub fn is_admin(&self) -> bool {
        matches!(self, Role::Admin)
    }

    /// Default role for new users (always User — first user is Admin).
    pub fn default_for_new_user() -> Self {
        Role::User
    }
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Permissions that can be granted to roles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Permission {
    /// View the admin dashboard.
    ViewDashboard,
    /// List all users.
    ListUsers,
    /// Change a user's role.
    ChangeUserRole,
    /// Delete a user.
    DeleteUser,
    /// Change project settings.
    ChangeSettings,
    /// View own profile.
    ViewOwnProfile,
    /// Edit own profile.
    EditOwnProfile,
}

impl Permission {
    /// Check if a role has this permission.
    pub fn granted_to(&self, role: Role) -> bool {
        match (role, self) {
            // Admin has all permissions.
            (Role::Admin, _) => true,
            // User can only view/edit own profile.
            (Role::User, Permission::ViewOwnProfile) => true,
            (Role::User, Permission::EditOwnProfile) => true,
            (Role::User, _) => false,
        }
    }

    /// Check if a role has a permission, returning an error if not.
    pub fn check(&self, role: Role) -> Result<(), RbacError> {
        if self.granted_to(role) {
            Ok(())
        } else {
            Err(RbacError::Denied {
                permission: format!("{:?}", self),
                role: role.to_string(),
            })
        }
    }
}

/// RBAC error.
#[derive(Debug, thiserror::Error)]
pub enum RbacError {
    #[error("permission denied: {permission} required, role={role}")]
    Denied { permission: String, role: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admin_has_all_permissions() {
        for perm in [
            Permission::ViewDashboard,
            Permission::ListUsers,
            Permission::ChangeUserRole,
            Permission::DeleteUser,
            Permission::ChangeSettings,
            Permission::ViewOwnProfile,
            Permission::EditOwnProfile,
        ] {
            assert!(perm.granted_to(Role::Admin), "admin should have {:?}", perm);
        }
    }

    #[test]
    fn user_has_limited_permissions() {
        assert!(Permission::ViewOwnProfile.granted_to(Role::User));
        assert!(Permission::EditOwnProfile.granted_to(Role::User));
        assert!(!Permission::ViewDashboard.granted_to(Role::User));
        assert!(!Permission::ListUsers.granted_to(Role::User));
        assert!(!Permission::ChangeUserRole.granted_to(Role::User));
        assert!(!Permission::DeleteUser.granted_to(Role::User));
        assert!(!Permission::ChangeSettings.granted_to(Role::User));
    }

    #[test]
    fn role_from_str() {
        assert_eq!(Role::from_str("admin"), Some(Role::Admin));
        assert_eq!(Role::from_str("user"), Some(Role::User));
        assert_eq!(Role::from_str("invalid"), None);
    }

    #[test]
    fn permission_check_returns_error() {
        let result = Permission::ViewDashboard.check(Role::User);
        assert!(result.is_err());
    }

    #[test]
    fn permission_check_admin_ok() {
        let result = Permission::ChangeSettings.check(Role::Admin);
        assert!(result.is_ok());
    }
}
