use super::{AccessAction, Actor, ProtectedResource};

pub struct AccessRequest {
    pub actor: Actor,
    pub target: ProtectedResource,
    pub action: AccessAction,
}
