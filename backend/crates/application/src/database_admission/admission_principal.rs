/// A stable identity supplied by a trusted application boundary.
///
/// Construct this from verified identity, never directly from request input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AdmissionPrincipal(u128);

impl AdmissionPrincipal {
    pub const fn from_trusted_id(id: u128) -> Self {
        Self(id)
    }
}
