#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompleteOutcome {
    Completed,
    LeaseNotCurrent,
}
