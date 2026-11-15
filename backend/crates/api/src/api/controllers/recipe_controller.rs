use std::sync::OnceLock;

use axum::{Json, http::StatusCode};
use orchestrata_recipes::load_recipe;

use crate::dto::recipe_metadata_response::RecipeMetadataResponse;

const BUILTIN_RECIPE_ID: &str = "field-service";
const BUILTIN_RECIPE_XML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../recipes/field-service.xml"
));
static FIELD_SERVICE: OnceLock<Option<RecipeMetadataResponse>> = OnceLock::new();

#[utoipa::path(
    get,
    path = "/api/v1/recipes/field-service",
    responses(
        (status = 200, description = "Field-service recipe input and output metadata", body = RecipeMetadataResponse),
        (status = 500, description = "Built-in recipe metadata is unavailable")
    )
)]
pub async fn field_service()
-> Result<Json<&'static RecipeMetadataResponse>, (StatusCode, &'static str)> {
    FIELD_SERVICE
        .get_or_init(load_field_service)
        .as_ref()
        .map(Json)
        .ok_or((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Built-in recipe metadata is unavailable",
        ))
}

fn load_field_service() -> Option<RecipeMetadataResponse> {
    let recipe = load_recipe(BUILTIN_RECIPE_XML).ok()?;
    if recipe.id() != BUILTIN_RECIPE_ID {
        return None;
    }
    Some((&recipe).into())
}

#[cfg(test)]
mod tests {
    use crate::dto::recipe_field_kind::RecipeFieldKind;

    use super::*;

    #[tokio::test]
    async fn returns_the_checked_in_field_service_contract() {
        let Json(metadata) = field_service().await.unwrap();

        assert_eq!(metadata.id, "field-service");
        assert_eq!(metadata.title, "Assign technicians to overlapping jobs");
        assert_eq!(metadata.inputs.len(), 4);
        assert_eq!(metadata.inputs[0].name, "job-names");
        assert_eq!(metadata.inputs[0].kind, RecipeFieldKind::StringList);
        assert!(metadata.inputs[0].required);
        assert!(!metadata.inputs[3].required);
        assert_eq!(metadata.outputs.len(), 1);
        assert_eq!(metadata.outputs[0].name, "assigned-technicians");
        assert_eq!(metadata.outputs[0].kind, RecipeFieldKind::IntegerList);
    }
}
