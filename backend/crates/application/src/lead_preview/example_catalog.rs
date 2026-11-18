use serde::Serialize;

/// Identifiers for the read-only, built-in lead preview.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct LeadExampleCatalog {
    pub synthetic: bool,
    pub lead_ids: Vec<String>,
}
