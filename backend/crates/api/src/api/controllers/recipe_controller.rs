use axum::{
    Json,
    extract::{Path, State},
    http::{StatusCode, header::CONTENT_TYPE},
};
use orchestrata_application::solve_error::SolveError;
use orchestrata_solver::Problem;

use crate::{
    api::{
        error::ApiError, error_code::SolveErrorCode, recipe_catalog,
        solve_admission::SolveAdmission,
    },
    dto::{recipe_metadata_response::RecipeMetadataResponse, solve_response::SolveResponse},
};

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

#[utoipa::path(
    post, path = "/api/v1/recipes/{recipe_id}/run-example",
    description = "Solve the checked-in synthetic example for this recipe. No request body or external effects; recipes with variants use the default example.",
    params(("recipe_id" = String, Path, description = "Built-in recipe identifier")),
    responses(
        (status = 200, description = "Verified result; solution is null if proven infeasible", body = SolveResponse),
        (status = 404, description = "Unknown built-in recipe",
            headers(("X-Orchestrata-Error-Code" = String, description = "not_found"))),
        (status = 500, description = "The server could not solve the built-in example",
            headers(("X-Orchestrata-Error-Code" = SolveErrorCode, description = "solve_execution_failed"))),
        (status = 503, description = "All solve execution slots are occupied; retry later",
            headers(("X-Orchestrata-Error-Code" = SolveErrorCode, description = "solver_busy")))
    )
)]
pub async fn run_example(
    State(admission): State<SolveAdmission>,
    Path(id): Path<String>,
) -> Result<Json<SolveResponse>, ApiError> {
    let source = recipe_catalog::find_example(&id).ok_or(ApiError::UnknownRecipe)?;
    let problem: Problem = serde_json::from_str(source).map_err(|_| SolveError::ExecutionFailed)?;
    let solution = admission
        .solve(problem)
        .await
        .map_err(|error| match error {
            ApiError::SolveBusy => ApiError::SolveBusy,
            _ => SolveError::ExecutionFailed.into(),
        })?;
    Ok(Json(SolveResponse { solution }))
}

#[cfg(test)]
#[path = "../../../../../../backend_tests/api/recipe_controller_unit.rs"]
mod tests;
