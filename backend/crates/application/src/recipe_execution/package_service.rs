use std::collections::BTreeMap;

use orchestrata_recipe_package::{DistributionTier, load_package};

use super::{
    InputIdentity, Objective, ObjectiveResult, PackageRegistration, PackageRun, PackageSelection,
    RecipeRunError, RunRequest, RunScope, SessionAuthority, registered_package::RegisteredPackage,
};

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
        let validated = load_package(bytes)?;
        let metadata = validated.metadata();
        if registration.workspace.is_empty()
            || metadata.id != registration.package_id
            || validated.content_identity().to_hex() != registration.content_revision
            || registration.entrypoints.is_empty()
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
