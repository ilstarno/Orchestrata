use axum::{
    Json,
    extract::Path,
    http::{StatusCode, header::CONTENT_TYPE},
};
use orchestrata_solver::Problem;

use crate::{api::recipe_catalog, dto::recipe_metadata_response::RecipeMetadataResponse};

const UNAVAILABLE: (StatusCode, &str) = (
    StatusCode::INTERNAL_SERVER_ERROR,
    "Built-in recipe metadata is unavailable",
);
const UNKNOWN_RECIPE: (StatusCode, &str) = (StatusCode::NOT_FOUND, "Unknown built-in recipe");

#[utoipa::path(
    get, path = "/api/v1/recipes",
    responses(
        (status = 200, description = "Included recipe catalog", body = [RecipeMetadataResponse]),
        (status = 500, description = "Built-in recipe metadata is unavailable")
    )
)]
pub async fn list() -> Result<Json<&'static [RecipeMetadataResponse]>, (StatusCode, &'static str)> {
    recipe_catalog::all().map(Json).ok_or(UNAVAILABLE)
}

#[utoipa::path(
    get, path = "/api/v1/recipes/{recipe_id}",
    params(("recipe_id" = String, Path, description = "Built-in recipe identifier")),
    responses(
        (status = 200, description = "Recipe input and output metadata", body = RecipeMetadataResponse),
        (status = 404, description = "Unknown built-in recipe"),
        (status = 500, description = "Built-in recipe metadata is unavailable")
    )
)]
pub async fn metadata(
    Path(id): Path<String>,
) -> Result<Json<&'static RecipeMetadataResponse>, (StatusCode, &'static str)> {
    recipe_catalog::all()
        .ok_or(UNAVAILABLE)?
        .iter()
        .find(|entry| entry.id == id)
        .map(Json)
        .ok_or(UNKNOWN_RECIPE)
}

#[utoipa::path(
    get, path = "/api/v1/recipes/{recipe_id}/example",
    params(("recipe_id" = String, Path, description = "Built-in recipe identifier")),
    responses(
        (status = 200, description = "Checked-in synthetic solver problem", body = Problem),
        (status = 404, description = "Unknown built-in recipe")
    )
)]
pub async fn example(
    Path(id): Path<String>,
) -> Result<([(axum::http::HeaderName, &'static str); 1], &'static str), (StatusCode, &'static str)>
{
    let problem = recipe_catalog::find_example(&id).ok_or(UNKNOWN_RECIPE)?;
    Ok(([(CONTENT_TYPE, "application/json")], problem))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn keeps_the_field_service_route_contract() {
        let Json(metadata) = metadata(Path("field-service".into())).await.unwrap();
        assert_eq!(metadata.id, "field-service");
        assert_eq!(metadata.inputs.len(), 4);
        assert_eq!(metadata.outputs[0].name, "assigned-technicians");
        assert_eq!(example(Path("missing".into())).await, Err(UNKNOWN_RECIPE));
    }
}
