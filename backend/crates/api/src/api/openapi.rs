use orchestrata_solver::{Constraint, Problem, Score, Solution, Variable};
use utoipa::OpenApi;

use crate::{
    api::{
        controllers::{
            error_catalog_controller, health_controller, recipe_controller, solve_controller,
        },
        error_code::SolveErrorCode,
    },
    dto::{
        error_catalog_entry::ErrorCatalogEntry, health_response::HealthResponse,
        recipe_field_kind::RecipeFieldKind, recipe_field_response::RecipeFieldResponse,
        recipe_metadata_response::RecipeMetadataResponse, solve_response::SolveResponse,
    },
};

#[derive(OpenApi)]
#[openapi(
    paths(
        error_catalog_controller::error_catalog,
        health_controller::health,
        recipe_controller::list,
        recipe_controller::metadata,
        recipe_controller::example,
        recipe_controller::run_example,
        solve_controller::solve_problem
    ),
    components(schemas(
        HealthResponse,
        SolveResponse,
        SolveErrorCode,
        ErrorCatalogEntry,
        RecipeMetadataResponse,
        RecipeFieldResponse,
        RecipeFieldKind,
        Problem,
        Variable,
        Constraint,
        Solution,
        Score
    ))
)]
pub struct ApiDoc;
