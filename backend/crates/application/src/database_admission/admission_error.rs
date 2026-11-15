use super::AdmissionLimit;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdmissionError {
    Busy(AdmissionLimit),
    WaitingFull(AdmissionLimit),
    TimedOut,
    Unavailable,
}
