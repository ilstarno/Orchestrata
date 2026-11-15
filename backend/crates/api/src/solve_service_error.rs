#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SolveServiceError {
    InvalidProblem,
    ExecutionFailed,
}
