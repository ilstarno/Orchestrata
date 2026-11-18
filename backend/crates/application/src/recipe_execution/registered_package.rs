use std::collections::BTreeMap;

use orchestrata_recipe_package::ValidatedPackage;

use super::Objective;

pub(super) struct RegisteredPackage {
    pub(super) validated: ValidatedPackage,
    pub(super) requires_entitlement: bool,
    pub(super) entrypoints: BTreeMap<String, (String, Objective)>,
}
