use orchestrata_application::access::{
    AccessAction::{Manage, Run, View},
    AccessRequest, Actor, ActorId, AgentId, GroupMembership, OrganizationId, Permission,
    PermissionGroup, ProtectedResource, ResourceOwner, UserId, is_allowed,
};

const ALICE: Actor = Actor::User(UserId(1));
const BOB: Actor = Actor::User(UserId(2));
const ALICE_PERSONAL: ResourceOwner = ResourceOwner::Personal(UserId(1));
const BOB_PERSONAL: ResourceOwner = ResourceOwner::Personal(UserId(2));
const FIRST_ORG: ResourceOwner = ResourceOwner::Organization(OrganizationId(1));
const SECOND_ORG: ResourceOwner = ResourceOwner::Organization(OrganizationId(2));

#[test]
fn groups_provide_a_union_within_the_matching_scope() {
    let analysts = PermissionGroup::new(FIRST_ORG, [Permission::Solver(View)]);
    let operators = PermissionGroup::new(FIRST_ORG, [Permission::Solver(Run)]);
    let memberships = [member(ALICE, &analysts), member(ALICE, &operators)];
    let solver = ProtectedResource::Solver(FIRST_ORG);

    assert!(is_allowed(request(ALICE, solver, View), &memberships));
    assert!(is_allowed(request(ALICE, solver, Run), &memberships));
    assert!(!is_allowed(request(ALICE, solver, Manage), &memberships));
}

#[test]
fn organization_memberships_do_not_cross_scopes_or_principals() {
    let group = PermissionGroup::new(FIRST_ORG, [Permission::Solver(Run)]);
    let memberships = [member(ALICE, &group)];

    assert!(!is_allowed(
        request(ALICE, ProtectedResource::Solver(SECOND_ORG), Run),
        &memberships,
    ));
    assert!(!is_allowed(
        request(BOB, ProtectedResource::Solver(FIRST_ORG), Run),
        &memberships,
    ));
}

#[test]
fn one_user_can_have_distinct_grants_in_different_organizations() {
    let readers = PermissionGroup::new(FIRST_ORG, [Permission::Solver(View)]);
    let operators = PermissionGroup::new(SECOND_ORG, [Permission::Solver(Run)]);
    let memberships = [member(ALICE, &readers), member(ALICE, &operators)];

    assert!(is_allowed(
        request(ALICE, ProtectedResource::Solver(FIRST_ORG), View),
        &memberships,
    ));
    assert!(is_allowed(
        request(ALICE, ProtectedResource::Solver(SECOND_ORG), Run),
        &memberships,
    ));
    assert!(!is_allowed(
        request(ALICE, ProtectedResource::Solver(FIRST_ORG), Run),
        &memberships,
    ));
}

#[test]
fn permissions_are_resource_and_action_specific() {
    let group = PermissionGroup::new(FIRST_ORG, [Permission::Agent(Manage)]);
    let memberships = [member(ALICE, &group)];

    assert!(is_allowed(
        request(ALICE, ProtectedResource::Agent(FIRST_ORG), Manage),
        &memberships,
    ));
    assert!(!is_allowed(
        request(ALICE, ProtectedResource::Agent(FIRST_ORG), Run),
        &memberships,
    ));
    assert!(!is_allowed(
        request(ALICE, ProtectedResource::Solver(FIRST_ORG), Manage),
        &memberships,
    ));
}

#[test]
fn people_manage_their_own_personal_resources_and_cannot_use_anothers() {
    let personal_group = PermissionGroup::new(
        BOB_PERSONAL,
        [Permission::Solver(Run), Permission::Agent(Run)],
    );
    let memberships = [member(ALICE, &personal_group)];

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
fn organization_resources_require_explicit_grants() {
    assert!(!is_allowed(
        request(ALICE, ProtectedResource::Agent(FIRST_ORG), View),
        &[],
    ));
}

#[test]
fn agent_has_its_own_grants_not_its_personal_owners_access() {
    let agent = Actor::Agent {
        id: AgentId(1),
        owner: ALICE_PERSONAL,
    };
    let group = PermissionGroup::new(ALICE_PERSONAL, [Permission::Solver(Run)]);
    let solver = ProtectedResource::Solver(ALICE_PERSONAL);

    assert!(!is_allowed(request(agent, solver, Run), &[]));
    assert!(!is_allowed(
        request(agent, solver, Run),
        &[member(ALICE, &group)],
    ));
    assert!(is_allowed(
        request(agent, solver, Run),
        &[member(agent, &group)],
    ));
    assert!(!is_allowed(
        request(agent, solver, Manage),
        &[member(agent, &group)],
    ));
}

#[test]
fn agent_cannot_use_even_an_explicit_group_outside_its_own_scope() {
    let agent = Actor::Agent {
        id: AgentId(2),
        owner: FIRST_ORG,
    };
    let first = PermissionGroup::new(FIRST_ORG, [Permission::Solver(Run)]);
    let second = PermissionGroup::new(SECOND_ORG, [Permission::Solver(Run)]);
    let personal = PermissionGroup::new(ALICE_PERSONAL, [Permission::Solver(Run)]);
    let memberships = [
        member(agent, &first),
        member(agent, &second),
        member(agent, &personal),
    ];

    assert!(is_allowed(
        request(agent, ProtectedResource::Solver(FIRST_ORG), Run),
        &memberships,
    ));
    assert!(!is_allowed(
        request(agent, ProtectedResource::Solver(SECOND_ORG), Run),
        &memberships,
    ));
    assert!(!is_allowed(
        request(agent, ProtectedResource::Solver(ALICE_PERSONAL), Run),
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
