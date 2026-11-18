#[path = "recipe_execution_oracle.rs"]
mod oracle;

use orchestrata_application::recipe_execution::{
    Objective, ObjectiveResult, PackageRegistration, PackageService, RecipeRunError, RunRequest,
    RunScope, SessionAuthority,
};

const PACKAGE: &[u8] = include_bytes!("fixtures/recipe_execution/maintenance.orcpkg");
const INPUT: &str = include_str!("fixtures/recipe_execution/maintenance.json");
const CONTENT: &str = "d386a7bb8878b432717300062a881739a99875a414848abb0207094bf5680639";
const COST: &str = "f85ceef657d9c954bc75fc4495641bef291d0ed3cb0e46e8982478e69d6eddd8";
const DELAY: &str = "27c907c4a3a0785d4e257b1c3047f71642189abc771753c0eb55c2013b77173e";
const PACKAGE_ID: &str = "maintenance-allocation";
const WORKSPACE: &str = "fixture-workspace";

struct FixtureAuthority;

impl SessionAuthority for FixtureAuthority {
    // In-process only; deliberately no wire conversion or public transport.
    type Session = bool;

    fn may_run(&self, session: &bool, scope: RunScope<'_>) -> bool {
        *session && scope.workspace == WORKSPACE
    }
}

fn registration(requires_entitlement: bool) -> PackageRegistration<'static> {
    PackageRegistration {
        workspace: WORKSPACE,
        package_id: PACKAGE_ID,
        content_revision: CONTENT,
        requires_entitlement,
        entrypoints: [
            (
                "cost".into(),
                (
                    COST.into(),
                    Objective::MinimizeEdgeSumAfterCoverage {
                        name: "Cost after coverage".into(),
                        unit: "fixture cost units".into(),
                    },
                ),
            ),
            (
                "delay".into(),
                (
                    DELAY.into(),
                    Objective::MinimizeEdgeSumAfterCoverage {
                        name: "Delay after coverage".into(),
                        unit: "fixture delay units".into(),
                    },
                ),
            ),
        ]
        .into(),
    }
}

fn service(requires_entitlement: bool) -> PackageService<FixtureAuthority> {
    let mut service = PackageService::new(FixtureAuthority);
    service
        .register(registration(requires_entitlement), PACKAGE)
        .unwrap();
    service
}

fn request<'a>(
    entrypoint: &'a str,
    execution_revision: &'a str,
    normalized_records: &'a str,
) -> RunRequest<'a> {
    RunRequest {
        scope: RunScope {
            workspace: WORKSPACE,
            package: PACKAGE_ID,
            entrypoint,
        },
        content_revision: CONTENT,
        execution_revision,
        normalized_records,
    }
}

#[test]
fn same_normalized_records_two_objectives_different_verified_plans() {
    let service = service(false);
    let cost = service.run(&true, request("cost", COST, INPUT)).unwrap();
    let delay = service.run(&true, request("delay", DELAY, INPUT)).unwrap();
    assert_eq!(cost.package_id, PACKAGE_ID);
    assert_eq!(cost.content_revision, CONTENT);
    assert_eq!(cost.execution_revision, COST);
    assert_eq!(delay.execution_revision, DELAY);
    oracle::check(INPUT, &cost, false);
    oracle::check(INPUT, &delay, true);
    assert_eq!(cost.input_identity, delay.input_identity);
    assert_eq!(
        cost.input_identity.as_str(),
        "bd280942a8d2e9381c7dc49a3ab749f3ee69b508a60bef1283a4443e24d4ad78"
    );
    assert_eq!(cost.solution.assigned, 3);
    assert_eq!(delay.solution.assigned, 3);
    assert_eq!(
        cost.solution.assignments[0].resource_id.as_deref(),
        Some("mech-east")
    );
    assert_eq!(
        delay.solution.assignments[0].resource_id.as_deref(),
        Some("mech-west")
    );
    assert_eq!(cost.solution.assignments[2].resource_id, None);
    assert_eq!(
        delay.solution.assignments[2].resource_id.as_deref(),
        Some("safety-day")
    );
    assert_eq!(
        cost.objective,
        ObjectiveResult::MinimizeEdgeSumAfterCoverage {
            name: "Cost after coverage".into(),
            value: 6,
            unit: "fixture cost units".into()
        }
    );
    assert_eq!(
        delay.objective,
        ObjectiveResult::MinimizeEdgeSumAfterCoverage {
            name: "Delay after coverage".into(),
            value: 6,
            unit: "fixture delay units".into()
        }
    );
}

#[test]
fn unverified_session_wrong_workspace_and_paid_classification_stop_before_bad_input() {
    let free = service(false);
    assert_eq!(
        free.run(&false, request("cost", COST, "not valid"))
            .unwrap_err(),
        RecipeRunError::Unauthorized
    );
    let wrong_workspace = RunRequest {
        scope: RunScope {
            workspace: "other-workspace",
            ..request("cost", COST, INPUT).scope
        },
        ..request("cost", COST, INPUT)
    };
    assert_eq!(
        free.run(&true, wrong_workspace).unwrap_err(),
        RecipeRunError::Unauthorized
    );
    assert_eq!(
        service(true)
            .run(&true, request("cost", COST, "not valid"))
            .unwrap_err(),
        RecipeRunError::EntitlementRequired
    );
}

#[test]
fn stale_package_and_execution_revisions_fail_before_compilation() {
    let service = service(false);
    assert_eq!(
        service
            .run(&true, request("cost", DELAY, "not valid"))
            .unwrap_err(),
        RecipeRunError::StaleRevision
    );
    let revised = RunRequest {
        content_revision: COST,
        ..request("cost", COST, "not valid")
    };
    assert_eq!(
        service.run(&true, revised).unwrap_err(),
        RecipeRunError::StaleRevision
    );
    assert_eq!(
        service
            .run(&true, request("unlisted", COST, INPUT))
            .unwrap_err(),
        RecipeRunError::UnknownEntrypoint
    );
}

#[test]
fn invalid_records_and_capacity_bound_are_distinct() {
    let service = service(false);
    let wrong_shape = r#"{"orders":[],"crew":[],"options":[{"task":"a","person":"b","price":42}]}"#;
    assert_eq!(
        service
            .run(&true, request("cost", COST, wrong_shape))
            .unwrap_err(),
        RecipeRunError::InvalidInput
    );
    let too_many_orders = (0..33)
        .map(|i| serde_json::json!({"key":format!("order-{i}")}))
        .collect::<Vec<_>>();
    let oversized =
        serde_json::json!({"orders":too_many_orders, "crew":[], "options":[]}).to_string();
    assert_eq!(
        service
            .run(&true, request("cost", COST, &oversized))
            .unwrap_err(),
        RecipeRunError::CapacityLimit
    );
}

#[test]
fn corrupted_or_mismatched_registration_does_not_enter_catalog() {
    let mut service = PackageService::new(FixtureAuthority);
    assert_eq!(
        service
            .register(registration(false), b"not a ZIP")
            .unwrap_err(),
        RecipeRunError::InvalidPackage
    );
    let wrong_revision = PackageRegistration {
        content_revision: DELAY,
        ..registration(false)
    };
    assert_eq!(
        service.register(wrong_revision, PACKAGE).unwrap_err(),
        RecipeRunError::InvalidPackage
    );
    assert_eq!(
        service
            .run(&true, request("cost", COST, INPUT))
            .unwrap_err(),
        RecipeRunError::UnknownPackage
    );
}

#[test]
fn published_maintenance_select_then_run_keeps_locale_out_of_execution_identity() {
    const PACKAGE: &[u8] = include_bytes!("fixtures/recipe_execution/maintenance-shift.orcpkg");
    const INPUT: &str = include_str!("fixtures/recipe_execution/maintenance-shift-inputs.json");
    const CONTENT: &str = "c491013d3969b76ce029dee4747f6b68368190ac44042a081649fb9b39eb5769";
    const COST: &str = "7aa894377f8c767ca41282b0dc269b4bd4697eea7b98254a834eaaaa811f59c3";
    const DELAY: &str = "734d2640cb2861840cdbf4d1b92c1fcef8e9f8bdd9b11c83d3467ada5b2f4965";
    const ID: &str = "maintenance-shift";
    let mut service = PackageService::new(FixtureAuthority);
    service
        .register(
            PackageRegistration {
                workspace: WORKSPACE,
                package_id: ID,
                content_revision: CONTENT,
                requires_entitlement: false,
                entrypoints: [
                    (
                        "cost".into(),
                        (
                            COST.into(),
                            Objective::MinimizeEdgeSumAfterCoverage {
                                name: "Cost after coverage".into(),
                                unit: "cents".into(),
                            },
                        ),
                    ),
                    (
                        "delay".into(),
                        (
                            DELAY.into(),
                            Objective::MinimizeEdgeSumAfterCoverage {
                                name: "Delay after coverage".into(),
                                unit: "minutes".into(),
                            },
                        ),
                    ),
                ]
                .into(),
            },
            PACKAGE,
        )
        .unwrap();
    let scope = |entrypoint| RunScope {
        workspace: WORKSPACE,
        package: ID,
        entrypoint,
    };
    let albanian = service.select(&true, scope("cost"), "sq").unwrap();
    let english = service
        .select(&true, scope("cost"), "unavailable-locale")
        .unwrap();
    let review_delay = service.select(&true, scope("delay"), "en").unwrap();
    assert_eq!(albanian.locale, "sq");
    assert_eq!(english.locale, "en");
    assert_ne!(albanian.title, english.title);
    assert_eq!(albanian.content_revision, english.content_revision);
    assert_eq!(albanian.execution_revision, english.execution_revision);
    assert_eq!(english.content_revision, review_delay.content_revision);
    assert_ne!(english.execution_revision, review_delay.execution_revision);
    assert_eq!(
        service.select(&false, scope("cost"), "sq").unwrap_err(),
        RecipeRunError::Unauthorized
    );
    let run = |scope, review: &orchestrata_application::recipe_execution::PackageSelection| {
        service
            .run(
                &true,
                RunRequest {
                    scope,
                    content_revision: &review.content_revision,
                    execution_revision: &review.execution_revision,
                    normalized_records: INPUT,
                },
            )
            .unwrap()
    };
    let cost = run(scope("cost"), &albanian);
    let delay = run(scope("delay"), &review_delay);
    assert_eq!(
        cost.input_identity.as_str(),
        "07cf573344aab46688919e5876dfcb81396bc4eef1b855cbaa5823de94de7477"
    );
    oracle::check(INPUT, &cost, false);
    oracle::check(INPUT, &delay, true);
    assert_eq!(cost.input_identity, delay.input_identity);
    assert_eq!(cost.solution.assigned, 10);
    assert_eq!(delay.solution.assigned, 10);
    assert_eq!(cost.solution.assignments.len(), 12);
    assert_eq!(delay.solution.assignments.len(), 12);
    assert_eq!(
        cost.solution.assignments[0].resource_id.as_deref(),
        Some("mechanical-west")
    );
    assert_eq!(
        delay.solution.assignments[0].resource_id.as_deref(),
        Some("mechanical-east")
    );
    assert_eq!(
        cost.objective,
        ObjectiveResult::MinimizeEdgeSumAfterCoverage {
            name: "Cost after coverage".into(),
            value: 14_600,
            unit: "cents".into()
        }
    );
    assert_eq!(
        delay.objective,
        ObjectiveResult::MinimizeEdgeSumAfterCoverage {
            name: "Delay after coverage".into(),
            value: 292,
            unit: "minutes".into()
        }
    );
}
