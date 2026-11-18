use std::collections::BTreeMap;

use orchestrata_application::recipe_execution::{
    Objective, PackageRegistration, PackageService, PenaltyEntrypointRegistration, PenaltyRunError,
    RecipeRunError, RunRequest, RunScope, SessionAuthority,
};

const MIXED: &[u8] = include_bytes!("fixtures/recipe_execution/synthetic-mixed.orcpkg");
const PAID: &[u8] = include_bytes!("fixtures/recipe_execution/synthetic-paid.orcpkg");
const UNKNOWN_MODEL: &[u8] =
    include_bytes!("fixtures/recipe_execution/synthetic-unknown-model.orcpkg");
const V2_INPUT: &str = include_str!("fixtures/recipe_execution/penalty-input.json");
const V1_INPUT: &str = include_str!("fixtures/recipe_execution/maintenance.json");
const PACKAGE: &str = "synthetic-mixed-approval";
const CONTENT: &str = "7371438604cbe38e3f1a211ec86a7092780106eb62a9e1cd7d567a24ff4a5fdf";
const PAID_CONTENT: &str = "cb2385c2bc3404235d46ffea3b4e3135d7269b26fc2dc476ce45a9816d6ddb63";
const V2: &str = "57c1a9ba6742b95d3f5d6cd867680c9cd7ce2cf90410155e2790997a47f9defd";
const V1: &str = "3756724d656d452b7a825c4b75802ef3a37b407875d5b234148e9756641016a0";

struct FixtureSessionAuthority;

impl SessionAuthority for FixtureSessionAuthority {
    type Session = bool;

    fn may_run(&self, session: &Self::Session, scope: RunScope<'_>) -> bool {
        *session && scope.workspace == "synthetic-workspace"
    }
}

fn registration(content: &str) -> PackageRegistration<'_> {
    PackageRegistration {
        workspace: "synthetic-workspace",
        package_id: PACKAGE,
        content_revision: content,
        requires_entitlement: false,
        entrypoints: [(
            "coverage".into(),
            (
                V1.into(),
                Objective::MinimizeEdgeSumAfterCoverage {
                    name: "Delay after coverage".into(),
                    unit: "fixture delay units".into(),
                },
            ),
        )]
        .into(),
    }
}

fn opts(id: &str, execution: &str, unit: &str) -> BTreeMap<String, PenaltyEntrypointRegistration> {
    [(
        id.into(),
        PenaltyEntrypointRegistration {
            execution_revision: execution.into(),
            expected_unit: unit.into(),
        },
    )]
    .into()
}

fn request<'a>(entrypoint: &'a str, records: &'a str) -> RunRequest<'a> {
    RunRequest {
        scope: RunScope {
            workspace: "synthetic-workspace",
            package: PACKAGE,
            entrypoint,
        },
        content_revision: CONTENT,
        execution_revision: if entrypoint == "penalty" { V2 } else { V1 },
        normalized_records: records,
    }
}

fn service(unit: &str) -> PackageService<FixtureSessionAuthority> {
    let mut service = PackageService::new(FixtureSessionAuthority);
    service
        .register_with_penalty_v2(registration(CONTENT), opts("penalty", V2, unit), MIXED)
        .unwrap();
    service
}

#[test]
fn explicit_v2_preserves_named_rows_and_cost_components_on_mixed_archive() {
    let service = service("cents");
    let result = service
        .run_penalty_v2(&true, request("penalty", V2_INPUT))
        .unwrap();
    assert_eq!(result.package_id, PACKAGE);
    assert_eq!(result.content_revision, CONTENT);
    assert_eq!(result.execution_revision, V2);
    assert_eq!(
        result.input_identity.as_str(),
        "6ae0dbac6f7137d32e1f94ede527ed856836dfa58b9cc7ebb008918a952f7d02"
    );
    assert_eq!(result.solution.schema_version, 2);
    assert_eq!(result.solution.objective, "service-plus-unassigned-cost");
    assert_eq!(result.solution.unit, "cents");
    assert_eq!(result.solution.assigned, 3);
    assert_eq!(result.solution.service_cost, 600);
    assert_eq!(result.solution.unassigned_cost, 13_100);
    assert_eq!(result.solution.total_cost, 13_700);
    let assignments = result
        .solution
        .assignments
        .iter()
        .map(|row| (row.item_id.as_str(), row.resource_id.as_deref()))
        .collect::<Vec<_>>();
    assert_eq!(
        assignments,
        [
            ("fraud", Some("generalist")),
            ("billing", None),
            ("routine", None),
            ("language-support", None),
            ("outage", Some("generalist")),
            ("compliance", Some("specialist")),
        ]
    );
    assert!(
        serde_json::to_value(&result).unwrap()["solution"]
            .get("minimized_edge_sum")
            .is_none()
    );
    let v1 = service.run(&true, request("coverage", V1_INPUT)).unwrap();
    assert_eq!(v1.execution_revision, V1);
    assert_eq!(v1.solution.minimized_edge_sum, Some(6));
    assert!(
        serde_json::to_value(&v1).unwrap()["solution"]
            .get("schema_version")
            .is_none()
    );
}

#[test]
fn explicit_runners_never_switch_family_or_infer_v2_opt_in() {
    let service = service("cents");
    assert_eq!(
        service
            .run(&true, request("penalty", V2_INPUT))
            .unwrap_err(),
        RecipeRunError::UnknownEntrypoint
    );
    assert_eq!(
        service
            .run_penalty_v2(&true, request("coverage", V1_INPUT))
            .unwrap_err(),
        PenaltyRunError::Package(RecipeRunError::UnknownEntrypoint),
    );
    let mut v1_only = PackageService::new(FixtureSessionAuthority);
    v1_only.register(registration(CONTENT), MIXED).unwrap();
    assert_eq!(
        v1_only
            .run_penalty_v2(&true, request("penalty", V2_INPUT))
            .unwrap_err(),
        PenaltyRunError::Package(RecipeRunError::UnknownEntrypoint),
    );
    let mut wrong_family = PackageService::new(FixtureSessionAuthority);
    let no_v1 = PackageRegistration {
        entrypoints: BTreeMap::new(),
        ..registration(CONTENT)
    };
    wrong_family
        .register_with_penalty_v2(no_v1, opts("coverage", V1, "cents"), MIXED)
        .unwrap();
    assert_eq!(
        wrong_family
            .run_penalty_v2(&true, request("coverage", V1_INPUT))
            .unwrap_err(),
        PenaltyRunError::UnsupportedModel,
    );
}

#[test]
fn authority_and_paid_gate_precede_parsing_and_revision_checks() {
    let service = service("cents");
    assert_eq!(
        service
            .run_penalty_v2(&false, request("penalty", "bad"))
            .unwrap_err(),
        PenaltyRunError::Package(RecipeRunError::Unauthorized),
    );
    let mut wrong_scope = request("penalty", "bad");
    wrong_scope.scope.workspace = "unrelated";
    assert_eq!(
        service.run_penalty_v2(&true, wrong_scope).unwrap_err(),
        PenaltyRunError::Package(RecipeRunError::Unauthorized),
    );
    let mut paid = PackageService::new(FixtureSessionAuthority);
    paid.register_with_penalty_v2(
        registration(PAID_CONTENT),
        opts("penalty", V2, "cents"),
        PAID,
    )
    .unwrap();
    assert_eq!(
        paid.run_penalty_v2(&true, request("penalty", "bad"))
            .unwrap_err(),
        PenaltyRunError::Package(RecipeRunError::EntitlementRequired),
    );
}

#[test]
fn expected_units_revisions_and_invalid_input_do_not_return_a_plan() {
    let correct = service("cents");
    let stale = RunRequest {
        execution_revision: V1,
        ..request("penalty", "bad")
    };
    assert_eq!(
        correct.run_penalty_v2(&true, stale).unwrap_err(),
        PenaltyRunError::Package(RecipeRunError::StaleRevision)
    );
    let stale_content = RunRequest {
        content_revision: PAID_CONTENT,
        ..request("penalty", "bad")
    };
    assert_eq!(
        correct.run_penalty_v2(&true, stale_content).unwrap_err(),
        PenaltyRunError::Package(RecipeRunError::StaleRevision)
    );
    assert_eq!(
        correct
            .run_penalty_v2(&true, request("penalty", V1_INPUT))
            .unwrap_err(),
        PenaltyRunError::Package(RecipeRunError::InvalidInput)
    );
    let too_large = " ".repeat(524_289);
    assert_eq!(
        correct
            .run_penalty_v2(&true, request("penalty", &too_large))
            .unwrap_err(),
        PenaltyRunError::Package(RecipeRunError::CapacityLimit),
    );
    let wrong = service("minutes");
    assert_eq!(
        wrong
            .run_penalty_v2(&true, request("penalty", V2_INPUT))
            .unwrap_err(),
        PenaltyRunError::UnitMismatch
    );
    let mut wrong_revision = PackageService::new(FixtureSessionAuthority);
    wrong_revision
        .register_with_penalty_v2(registration(CONTENT), opts("penalty", V1, "cents"), MIXED)
        .unwrap();
    let stale_registry = RunRequest {
        execution_revision: V1,
        ..request("penalty", V2_INPUT)
    };
    assert_eq!(
        wrong_revision
            .run_penalty_v2(&true, stale_registry)
            .unwrap_err(),
        PenaltyRunError::Package(RecipeRunError::StaleRevision)
    );
}

#[test]
fn unknown_model_schema_and_double_registered_entrypoint_reject_without_installing() {
    let mut service = PackageService::new(FixtureSessionAuthority);
    assert_eq!(
        service
            .register_with_penalty_v2(
                registration(CONTENT),
                opts("penalty", V2, "cents"),
                UNKNOWN_MODEL
            )
            .unwrap_err(),
        PenaltyRunError::UnsupportedPackage,
    );
    assert_eq!(
        service
            .register_with_penalty_v2(registration(CONTENT), opts("coverage", V1, "cents"), MIXED)
            .unwrap_err(),
        PenaltyRunError::Package(RecipeRunError::InvalidPackage),
    );
    assert_eq!(
        service
            .run_penalty_v2(&true, request("penalty", V2_INPUT))
            .unwrap_err(),
        PenaltyRunError::Package(RecipeRunError::UnknownPackage),
    );
}
