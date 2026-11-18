use super::{ActorId, AgentId, ResourceOwner, UserId};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Actor {
    User(UserId),
    Agent { id: AgentId, owner: ResourceOwner },
}

impl Actor {
    pub fn id(self) -> ActorId {
        match self {
            Self::User(id) => ActorId::User(id),
            Self::Agent { id, .. } => ActorId::Agent(id),
        }
    }
}
