use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct ErrorCatalogEntry {
    pub code: &'static str,
    pub description: &'static str,
    #[schema(value_type = Vec<u16>)]
    pub status_codes: &'static [u16],
}

impl ErrorCatalogEntry {
    pub const fn new(
        code: &'static str,
        description: &'static str,
        status_codes: &'static [u16],
    ) -> Self {
        Self {
            code,
            description,
            status_codes,
        }
    }
}
