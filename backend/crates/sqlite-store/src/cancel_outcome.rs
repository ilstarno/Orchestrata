#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CancelOutcome {
    Cancelled,
    AlreadyCancelled,
    NotPending,
    NotFound,
}
