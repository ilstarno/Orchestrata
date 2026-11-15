use super::{Permission, ResourceOwner};

pub struct PermissionGroup {
    owner: ResourceOwner,
    permissions: Vec<Permission>,
}

impl PermissionGroup {
    pub fn new(owner: ResourceOwner, permissions: impl IntoIterator<Item = Permission>) -> Self {
        Self {
            owner,
            permissions: permissions.into_iter().collect(),
        }
    }

    pub(super) fn grants(&self, owner: ResourceOwner, permission: Permission) -> bool {
        self.owner == owner && self.permissions.contains(&permission)
    }
}
