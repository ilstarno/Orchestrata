use orchestrata_recipe_runtime::mathematical::ModelSolution;
use serde::Serialize;

use super::{InputIdentity, ObjectiveResult};

#[derive(Debug, Serialize)]
pub struct PackageRun {
    pub package_id: String,
    pub entrypoint: String,
    pub content_revision: String,
    pub execution_revision: String,
    /// SHA-256 of the exact normalized UTF-8 bytes submitted to this run.
    pub input_identity: InputIdentity,
    pub objective: ObjectiveResult,
    pub solution: ModelSolution,
}
