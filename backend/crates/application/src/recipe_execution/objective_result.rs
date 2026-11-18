use serde::Serialize;

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ObjectiveResult {
    MaximizeCoverage {
        value: usize,
        unit: &'static str,
    },
    MinimizeEdgeSumAfterCoverage {
        name: String,
        value: u64,
        unit: String,
    },
}
