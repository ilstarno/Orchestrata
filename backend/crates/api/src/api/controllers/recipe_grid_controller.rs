use axum::{
    Json,
    extract::{Query, rejection::QueryRejection},
};

use crate::{
    api::{error::ApiError, recipe_catalog, recipe_grid},
    dto::{recipe_grid_query::RecipeGridQuery, recipe_grid_response::RecipeGridResponse},
};

#[utoipa::path(
    get, path = "/api/v1/recipes/grid",
    description = "Browse metadata for the included synthetic recipe catalog. Filtering, sorting and paging are performed on the server; no user records or uploaded data are used.",
    params(RecipeGridQuery),
    responses(
        (status = 200, description = "Recipe metadata columns and requested page", body = RecipeGridResponse),
        (status = 400, description = "Unknown, duplicate, malformed or out-of-bounds query parameter", body = String, content_type = "text/plain",
            headers(("X-Orchestrata-Error-Code" = String, description = "request_rejected"))),
        (status = 500, description = "Built-in recipe metadata is unavailable", body = String, content_type = "text/plain",
            headers(("X-Orchestrata-Error-Code" = String, description = "internal_error")))
    )
)]
pub async fn grid(
    query: Result<Query<RecipeGridQuery>, QueryRejection>,
) -> Result<Json<RecipeGridResponse>, ApiError> {
    let Query(query) = query.map_err(|_| ApiError::InvalidRecipeGridQuery)?;
    if !query.is_valid() {
        return Err(ApiError::InvalidRecipeGridQuery);
    }
    let catalog = recipe_catalog::all().ok_or(ApiError::RecipeCatalogUnavailable)?;
    Ok(Json(recipe_grid::page(query, catalog)))
}
