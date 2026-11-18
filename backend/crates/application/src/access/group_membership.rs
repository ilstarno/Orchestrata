use super::{ActorId, PermissionGroup};

pub struct GroupMembership<'a> {
    pub actor: ActorId,
    pub group: &'a PermissionGroup,
}
