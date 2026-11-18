#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdmissionLimit {
    Global,
    Principal,
}
