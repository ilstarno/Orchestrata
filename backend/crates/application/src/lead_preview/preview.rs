use serde::Serialize;

use super::{Outcome, Reason};

#[derive(Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct LeadPreview {
    pub lead_id: String,
    pub outcome: Outcome,
    pub reason: Reason,
    #[cfg_attr(feature = "openapi", schema(required = true))]
    pub assigned_representative_id: Option<String>,
    #[cfg_attr(feature = "openapi", schema(required = true))]
    pub contact_id: Option<String>,
}
