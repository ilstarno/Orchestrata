use super::{AccessAction, Permission, ResourceOwner};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProtectedResource {
    Solver(ResourceOwner),
    Agent(ResourceOwner),
}

impl ProtectedResource {
    pub fn owner(self) -> ResourceOwner {
        match self {
            Self::Solver(owner) | Self::Agent(owner) => owner,
        }
    }

    pub(super) fn permission_for(self, action: AccessAction) -> Permission {
        match self {
            Self::Solver(_) => Permission::Solver(action),
            Self::Agent(_) => Permission::Agent(action),
        }
    }
}
