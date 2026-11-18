/// Approved display metadata pinned to a specific Engine execution revision.
/// This does not supply or alter mathematical rules.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Objective {
    MaximizeCoverage,
    MinimizeEdgeSumAfterCoverage { name: String, unit: String },
}
