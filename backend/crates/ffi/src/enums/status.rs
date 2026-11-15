#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrcStatus {
    Ok = 0,
    InvalidArgument = 1,
    InvalidProblem = 2,
    ExecutionFailed = 3,
    BufferTooSmall = 4,
    InternalError = 5,
}
