use serde::Serialize;
use utoipa::ToSchema;

use super::{recipe_grid_column::RecipeGridColumn, recipe_grid_row::RecipeGridRow};

#[derive(Serialize, ToSchema)]
pub struct RecipeGridResponse {
    pub columns: [RecipeGridColumn; 3],
    pub items: Vec<RecipeGridRow>,
    #[schema(minimum = 1, maximum = 100000)]
    pub page: u32,
    #[schema(minimum = 1, maximum = 100)]
    pub page_size: u32,
    pub total_items: usize,
}
