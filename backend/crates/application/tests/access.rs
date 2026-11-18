use orchestrata_application::access::{
    AccessAction::{Manage, Run, View},
    AccessRequest, Actor, ActorId, AgentId, GroupMembership, Permission, PermissionGroup,
    ProtectedResource, ResourceOwner, UserId, is_allowed,
};

const ALICE: Actor = Actor::User(UserId(1));
const BOB: Actor = Actor::User(UserId(2));
const ALICE_PERSONAL: ResourceOwner = ResourceOwner::Personal(UserId(1));
const BOB_PERSONAL: ResourceOwner = ResourceOwner::Personal(UserId(2));
const SHARED: ResourceOwner = ResourceOwner::Shared;

#[test]
fn multiple_shared_groups_provide_a_union_of_explicit_grants() {
    let analysts = PermissionGroup::new(SHARED, [Permission::Solver(View)]);
    let operators = PermissionGroup::new(SHARED, [Permission::Solver(Run)]);
    let memberships = [member(ALICE, &analysts), member(ALICE, &operators)];
    let solver = ProtectedResource::Solver(SHARED);

    assert!(is_allowed(request(ALICE, solver, View), &memberships));
    assert!(is_allowed(request(ALICE, solver, Run), &memberships));
    assert!(!is_allowed(request(ALICE, solver, Manage), &memberships));
    assert!(!is_allowed(request(BOB, solver, Run), &memberships));
}

#[test]
fn shared_resources_default_to_deny_even_with_personal_grants() {
    let personal = PermissionGroup::new(ALICE_PERSONAL, [Permission::Solver(Run)]);
    let solver = ProtectedResource::Solver(SHARED);

    assert!(!is_allowed(request(ALICE, solver, Run), &[]));
    assert!(!is_allowed(
        request(ALICE, solver, Run),
        &[member(ALICE, &personal)],
    ));
}

#[test]
fn permissions_are_resource_and_action_specific() {
    let group = PermissionGroup::new(SHARED, [Permission::Agent(Manage)]);
    let memberships = [member(ALICE, &group)];

    assert!(is_allowed(
        request(ALICE, ProtectedResource::Agent(SHARED), Manage),
        &memberships,
    ));
    assert!(!is_allowed(
        request(ALICE, ProtectedResource::Agent(SHARED), Run),
        &memberships,
    ));
    assert!(!is_allowed(
        request(ALICE, ProtectedResource::Solver(SHARED), Manage),
        &memberships,
    ));
}

#[test]
fn personal_user_ownership_does_not_transfer_via_groups() {
    let bobs_personal_group = PermissionGroup::new(
        BOB_PERSONAL,
        [Permission::Solver(Run), Permission::Agent(Run)],
    );
    let shared_group = PermissionGroup::new(SHARED, [Permission::Solver(Run)]);
    let memberships = [
        member(ALICE, &bobs_personal_group),
        member(ALICE, &shared_group),
    ];

    assert!(is_allowed(
        request(ALICE, ProtectedResource::Solver(ALICE_PERSONAL), Manage),
        &[],
    ));
    assert!(is_allowed(
        request(BOB, ProtectedResource::Agent(BOB_PERSONAL), View),
        &[],
    ));
    assert!(!is_allowed(
        request(ALICE, ProtectedResource::Solver(BOB_PERSONAL), Run),
        &memberships,
    ));
    assert!(!is_allowed(
        request(ALICE, ProtectedResource::Agent(BOB_PERSONAL), Run),
        &memberships,
    ));
}

#[test]
fn a_personal_agent_needs_its_own_grant_not_its_owners() {
    let agent = Actor::Agent {
        id: AgentId(1),
        owner: ALICE_PERSONAL,
    };
    let personal = PermissionGroup::new(ALICE_PERSONAL, [Permission::Solver(Run)]);
    let solver = ProtectedResource::Solver(ALICE_PERSONAL);

    assert!(!is_allowed(request(agent, solver, Run), &[]));
    // Matching numeric IDs do not make a user and an agent the same principal.
    assert!(!is_allowed(
        request(agent, solver, Run),
        &[member(ALICE, &personal)],
    ));
    assert!(is_allowed(
        request(agent, solver, Run),
        &[member(agent, &personal)],
    ));
    assert!(!is_allowed(
        request(agent, solver, Manage),
        &[member(agent, &personal)],
    ));
}

#[test]
fn a_shared_agent_needs_an_explicit_shared_grant() {
    let agent = Actor::Agent {
        id: AgentId(2),
        owner: SHARED,
    };
    let shared = PermissionGroup::new(SHARED, [Permission::Solver(Run)]);
    let personal = PermissionGroup::new(ALICE_PERSONAL, [Permission::Solver(Run)]);
    let solver = ProtectedResource::Solver(SHARED);

    assert!(!is_allowed(request(agent, solver, Run), &[]));
    assert!(!is_allowed(
        request(agent, solver, Run),
        &[member(agent, &personal)],
    ));
    assert!(is_allowed(
        request(agent, solver, Run),
        &[member(agent, &shared)],
    ));
    assert!(!is_allowed(
        request(agent, ProtectedResource::Solver(ALICE_PERSONAL), Run),
        &[member(agent, &personal), member(agent, &shared)],
    ));
}

#[test]
fn a_personal_agent_cannot_cross_to_shared_or_another_personal_scope() {
    let agent = Actor::Agent {
        id: AgentId(3),
        owner: ALICE_PERSONAL,
    };
    let shared = PermissionGroup::new(SHARED, [Permission::Solver(Run)]);
    let other_personal = PermissionGroup::new(BOB_PERSONAL, [Permission::Solver(Run)]);
    let memberships = [member(agent, &shared), member(agent, &other_personal)];

    assert!(!is_allowed(
        request(agent, ProtectedResource::Solver(SHARED), Run),
        &memberships,
    ));
    assert!(!is_allowed(
        request(agent, ProtectedResource::Solver(BOB_PERSONAL), Run),
        &memberships,
    ));
}

fn member(actor: Actor, group: &PermissionGroup) -> GroupMembership<'_> {
    let actor: ActorId = actor.id();
    GroupMembership { actor, group }
}

fn request(
    actor: Actor,
    target: ProtectedResource,
    action: orchestrata_application::access::AccessAction,
) -> AccessRequest {
    AccessRequest {
        actor,
        target,
        action,
    }
}
