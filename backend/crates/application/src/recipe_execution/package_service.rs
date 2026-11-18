use std::collections::BTreeMap;

use orchestrata_recipe_package::{DistributionTier, ValidatedPackage, load_package};

use super::{
    InputIdentity, Objective, ObjectiveResult, PackageRegistration, PackageRun, PackageSelection,
    PenaltyEntrypointRegistration, PenaltyPackageRun, PenaltyRunError, RecipeRunError, RunRequest,
    RunScope, SessionAuthority, registered_package::RegisteredPackage,
};

// Keep numeric JSON exact if later models support larger values.
const MAX_EXACT_JSON_INTEGER: u64 = (1 << 53) - 1;

pub struct PackageService<A> {
    authority: A,
    packages: BTreeMap<String, BTreeMap<String, RegisteredPackage>>,
}

impl<A: SessionAuthority> PackageService<A> {
    pub fn new(authority: A) -> Self {
        Self {
            authority,
            packages: BTreeMap::new(),
        }
    }

    /// Register trusted configured bytes. No end-user upload or HTTP path calls this.
    pub fn register(
        &mut self,
        registration: PackageRegistration<'_>,
        bytes: &[u8],
    ) -> Result<(), RecipeRunError> {
        self.register_models(registration, BTreeMap::new(), load_package(bytes)?)
    }

    /// Explicit opt-in. A ZIP may contain both families, but an entrypoint is
    /// registered at most once, and neither run method calls the other.
    pub fn register_with_penalty_v2(
        &mut self,
        registration: PackageRegistration<'_>,
        penalty_entrypoints: BTreeMap<String, PenaltyEntrypointRegistration>,
        bytes: &[u8],
    ) -> Result<(), PenaltyRunError> {
        if penalty_entrypoints.is_empty() {
            return Err(RecipeRunError::InvalidPackage.into());
        }
        self.register_models(registration, penalty_entrypoints, load_package(bytes)?)?;
        Ok(())
    }

    fn register_models(
        &mut self,
        registration: PackageRegistration<'_>,
        penalty_entrypoints: BTreeMap<String, PenaltyEntrypointRegistration>,
        validated: ValidatedPackage,
    ) -> Result<(), RecipeRunError> {
        let metadata = validated.metadata();
        if registration.workspace.is_empty()
            || metadata.id != registration.package_id
            || validated.content_identity().to_hex() != registration.content_revision
            || (registration.entrypoints.is_empty() && penalty_entrypoints.is_empty())
            || penalty_entrypoints.iter().any(|(id, registration_v2)| {
                registration.entrypoints.contains_key(id)
                    || !validated.entrypoints().any(|entry| entry.id == *id)
                    || !is_digest(&registration_v2.execution_revision)
                    || registration_v2.expected_unit.trim().is_empty()
            })
            || registration.entrypoints.iter().any(|(id, (revision, objective))| {
                !validated.entrypoints().any(|entry| entry.id == *id)
                    || !is_digest(revision)
                    || matches!(objective, Objective::MinimizeEdgeSumAfterCoverage { name, unit } if name.trim().is_empty() || unit.trim().is_empty())
            })
        {
            return Err(RecipeRunError::InvalidPackage);
        }

        let requires_entitlement =
            registration.requires_entitlement || metadata.distribution == DistributionTier::Paid;
        let workspace = self
            .packages
            .entry(registration.workspace.to_owned())
            .or_default();
        if workspace.contains_key(registration.package_id) {
            return Err(RecipeRunError::InvalidPackage);
        }
        workspace.insert(
            registration.package_id.to_owned(),
            RegisteredPackage {
                validated,
                requires_entitlement,
                entrypoints: registration.entrypoints,
                penalty_entrypoints,
            },
        );
        Ok(())
    }

    pub fn select(
        &self,
        session: &A::Session,
        scope: RunScope<'_>,
        preferred_locale: &str,
    ) -> Result<PackageSelection, RecipeRunError> {
        let package = self.authorize_package(session, scope)?;
        let (revision, objective) = package
            .entrypoints
            .get(scope.entrypoint)
            .ok_or(RecipeRunError::UnknownEntrypoint)?;
        let meta = package.validated.metadata();
        let (locale, copy) = meta
            .translations
            .get_key_value(preferred_locale)
            .or_else(|| meta.translations.get_key_value(&meta.default_locale))
            .ok_or(RecipeRunError::InvalidPackage)?;
        Ok(PackageSelection {
            package_id: meta.id.clone(),
            entrypoint: scope.entrypoint.to_owned(),
            content_revision: package.validated.content_identity().to_hex(),
            execution_revision: revision.clone(),
            locale: locale.clone(),
            title: copy.title.clone(),
            description: copy.description.clone(),
            objective: objective.clone(),
        })
    }

    pub fn run(
        &self,
        session: &A::Session,
        request: RunRequest<'_>,
    ) -> Result<PackageRun, RecipeRunError> {
        let scope = request.scope;
        let package = self.authorize_package(session, scope)?;
        let (expected_execution, objective) = package
            .entrypoints
            .get(scope.entrypoint)
            .ok_or(RecipeRunError::UnknownEntrypoint)?;
        let content_revision = package.validated.content_identity().to_hex();
        if request.content_revision != content_revision
            || request.execution_revision != expected_execution
        {
            return Err(RecipeRunError::StaleRevision);
        }
        let compiled = package
            .validated
            .compile(Some(scope.entrypoint), request.normalized_records)?;
        let execution_revision = compiled.execution_identity().to_hex();
        if execution_revision != *expected_execution {
            return Err(RecipeRunError::StaleRevision);
        }
        let result = compiled.solve()?;
        let objective = match (objective, result.solution.minimized_edge_sum) {
            (Objective::MaximizeCoverage, None) => ObjectiveResult::MaximizeCoverage {
                unit: "items",
                value: result.solution.assigned,
            },
            (Objective::MinimizeEdgeSumAfterCoverage { name, unit }, Some(value)) => {
                ObjectiveResult::MinimizeEdgeSumAfterCoverage {
                    name: name.clone(),
                    unit: unit.clone(),
                    value,
                }
            }
            _ => return Err(RecipeRunError::InvalidPackage),
        };
        let input_identity = InputIdentity::from_records(request.normalized_records);
        Ok(PackageRun {
            package_id: scope.package.to_owned(),
            entrypoint: scope.entrypoint.to_owned(),
            content_revision,
            execution_revision,
            input_identity,
            objective,
            solution: result.solution,
        })
    }

    /// Compile and run only the registered v2 family. The v1 service and
    /// result body are unchanged; no automatic version fallback is attempted.
    pub fn run_penalty_v2(
        &self,
        session: &A::Session,
        request: RunRequest<'_>,
    ) -> Result<PenaltyPackageRun, PenaltyRunError> {
        let scope = request.scope;
        let package = self.authorize_package(session, scope)?;
        let registration = package
            .penalty_entrypoints
            .get(scope.entrypoint)
            .ok_or(RecipeRunError::UnknownEntrypoint)?;
        let content_revision = package.validated.content_identity().to_hex();
        if request.content_revision != content_revision
            || request.execution_revision != registration.execution_revision
        {
            return Err(RecipeRunError::StaleRevision.into());
        }
        let compiled = package
            .validated
            .compile_penalty(Some(scope.entrypoint), request.normalized_records)?;
        let execution_revision = compiled.execution_identity().to_hex();
        if execution_revision != registration.execution_revision {
            return Err(RecipeRunError::StaleRevision.into());
        }
        let result = compiled.solve()?.solution;
        if result.unit != registration.expected_unit {
            return Err(PenaltyRunError::UnitMismatch);
        }
        if result.schema_version != 2
            || result.objective != "service-plus-unassigned-cost"
            || result.service_cost.checked_add(result.unassigned_cost) != Some(result.total_cost)
            || result.total_cost > MAX_EXACT_JSON_INTEGER
            || result.assigned
                != result
                    .assignments
                    .iter()
                    .filter(|a| a.resource_id.is_some())
                    .count()
        {
            return Err(PenaltyRunError::InvalidResult);
        }
        Ok(PenaltyPackageRun {
            package_id: scope.package.to_owned(),
            entrypoint: scope.entrypoint.to_owned(),
            content_revision,
            execution_revision,
            input_identity: InputIdentity::from_records(request.normalized_records),
            solution: result,
        })
    }

    fn authorize_package(
        &self,
        session: &A::Session,
        scope: RunScope<'_>,
    ) -> Result<&RegisteredPackage, RecipeRunError> {
        if !self.authority.may_run(session, scope) {
            return Err(RecipeRunError::Unauthorized);
        }
        let package = self
            .packages
            .get(scope.workspace)
            .and_then(|workspace| workspace.get(scope.package))
            .ok_or(RecipeRunError::UnknownPackage)?;
        // No verified entitlement provider is wired. Even a free declaration
        // cannot authorize a package classified as paid by the installation.
        if package.requires_entitlement {
            return Err(RecipeRunError::EntitlementRequired);
        }
        Ok(package)
    }
}

fn is_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
