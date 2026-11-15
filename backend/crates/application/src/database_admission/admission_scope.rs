/// An authenticated principal within a tenant. Callers must supply trusted stable IDs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AdmissionScope {
    pub tenant: u128,
    pub principal: u128,
}
