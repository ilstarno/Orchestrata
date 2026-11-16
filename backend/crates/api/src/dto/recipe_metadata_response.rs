use orchestrata_recipes::Recipe;
use serde::Serialize;
use utoipa::ToSchema;

use super::recipe_field_response::RecipeFieldResponse;

#[derive(Serialize, ToSchema)]
pub struct RecipeMetadataResponse {
    pub id: String,
    pub title: String,
    pub scenario: String,
    pub inputs: Vec<RecipeFieldResponse>,
    pub outputs: Vec<RecipeFieldResponse>,
}

impl From<&Recipe> for RecipeMetadataResponse {
    fn from(recipe: &Recipe) -> Self {
        Self {
            id: recipe.id().to_owned(),
            title: recipe.title().to_owned(),
            scenario: recipe.scenario().to_owned(),
            inputs: recipe.inputs().iter().map(Into::into).collect(),
            outputs: recipe.outputs().iter().map(Into::into).collect(),
        }
    }
}
