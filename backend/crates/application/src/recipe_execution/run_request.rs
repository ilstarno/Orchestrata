use super::RunScope;

/// Records are the normalized JSON input to the selected Engine model,
/// regardless of whether they originated in a connector, upload or form.
pub struct RunRequest<'a> {
    pub scope: RunScope<'a>,
    pub content_revision: &'a str,
    pub execution_revision: &'a str,
    pub normalized_records: &'a str,
}
