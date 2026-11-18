use super::{AgentId, UserId};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActorId {
    User(UserId),
    Agent(AgentId),
}
