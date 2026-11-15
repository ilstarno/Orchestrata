#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdmissionLimit {
    Total,
    Tenant,
    Principal,
}
