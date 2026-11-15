use super::{AccessRequest, Actor, GroupMembership, ResourceOwner};

/// Evaluates access using authenticated identity, authoritative resource ownership, and current
/// group memberships. Resolve these inputs server-side instead of trusting request data.
/// This policy grants scope-wide access only; it does not authorize group administration.
#[must_use]
pub fn is_allowed(request: AccessRequest, memberships: &[GroupMembership<'_>]) -> bool {
    let owner = request.target.owner();
    match (request.actor, owner) {
        (Actor::User(actor), ResourceOwner::Personal(resource_owner)) => {
            return actor == resource_owner;
        }
        (
            Actor::Agent {
                owner: actor_owner, ..
            },
            _,
        ) if actor_owner != owner => {
            return false;
        }
        _ => {}
    }

    let actor = request.actor.id();
    let required = request.target.permission_for(request.action);
    memberships
        .iter()
        .any(|membership| membership.actor == actor && membership.group.grants(owner, required))
}
