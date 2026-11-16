use orchestrata_recipes::FieldSpec;
use serde::Serialize;
use utoipa::ToSchema;

use super::recipe_field_kind::RecipeFieldKind;

#[derive(Serialize, ToSchema)]
pub struct RecipeFieldResponse {
    pub name: String,
    pub kind: RecipeFieldKind,
    pub required: bool,
    pub description: String,
}

impl From<&FieldSpec> for RecipeFieldResponse {
    fn from(field: &FieldSpec) -> Self {
        Self {
            name: field.name.clone(),
            kind: field.kind.into(),
            required: field.required,
            description: field.description.clone(),
        }
    }
}
