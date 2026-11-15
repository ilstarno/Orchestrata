#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SolveError {
    InvalidInput,
    SearchLimitExceeded,
    ExecutionFailed,
}
