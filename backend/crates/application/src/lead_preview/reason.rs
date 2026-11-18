use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum Reason {
    New,
    Existing,
    Unqualified,
    Ambiguous,
    Invalid,
    Unassigned,
}
