use orchestrata_recipe_runtime::mathematical::PenaltyModelSolution;
use serde::Serialize;

use super::InputIdentity;

/// Separate result family. It has no alias for a v1 objective or score.
#[derive(Debug, Serialize)]
pub struct PenaltyPackageRun {
    pub package_id: String,
    pub entrypoint: String,
    pub content_revision: String,
    pub execution_revision: String,
    pub input_identity: InputIdentity,
    pub solution: PenaltyModelSolution,
}
