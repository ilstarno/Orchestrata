use super::{Permission, ResourceOwner};

/// A set of explicit grants for one resource scope in this installation.
pub struct PermissionGroup {
    scope: ResourceOwner,
    permissions: Vec<Permission>,
}

impl PermissionGroup {
    pub fn new(scope: ResourceOwner, permissions: impl IntoIterator<Item = Permission>) -> Self {
        Self {
            scope,
            permissions: permissions.into_iter().collect(),
        }
    }

    pub(super) fn grants(&self, scope: ResourceOwner, permission: Permission) -> bool {
        self.scope == scope && self.permissions.contains(&permission)
    }
}
