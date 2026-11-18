use orchestrata_core::ids::job_id::JobId;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SubmitOutcome {
    Created(JobId),
    Existing(JobId),
    ConflictingInput,
}
