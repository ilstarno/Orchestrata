use serde::Serialize;
use utoipa::ToSchema;

#[derive(Clone, Copy, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum RecipeGridColumnId {
    Id,
    Title,
    Scenario,
}
