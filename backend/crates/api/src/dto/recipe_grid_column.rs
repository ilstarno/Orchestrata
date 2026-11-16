use serde::Serialize;
use utoipa::ToSchema;

use super::recipe_grid_column_id::RecipeGridColumnId;

#[derive(Clone, Copy, Serialize, ToSchema)]
pub struct RecipeGridColumn {
    pub id: RecipeGridColumnId,
    pub label: &'static str,
    pub sortable: bool,
}
