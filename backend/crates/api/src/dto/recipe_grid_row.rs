use serde::Serialize;
use utoipa::ToSchema;

use super::recipe_metadata_response::RecipeMetadataResponse;

#[derive(Serialize, ToSchema)]
pub struct RecipeGridRow {
    pub id: &'static str,
    pub title: &'static str,
    pub scenario: &'static str,
}

impl From<&'static RecipeMetadataResponse> for RecipeGridRow {
    fn from(recipe: &'static RecipeMetadataResponse) -> Self {
        Self {
            id: &recipe.id,
            title: &recipe.title,
            scenario: &recipe.scenario,
        }
    }
}
