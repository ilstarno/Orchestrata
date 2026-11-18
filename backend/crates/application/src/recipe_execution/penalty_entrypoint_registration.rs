/// Trusted installation opt-in for one exact v2 model revision.
pub struct PenaltyEntrypointRegistration {
    pub execution_revision: String,
    /// Checked against the Engine's XML-bound unit before returning a result.
    pub expected_unit: String,
}
