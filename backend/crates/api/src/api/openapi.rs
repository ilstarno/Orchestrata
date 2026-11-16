use orchestrata_solver::{Constraint, Problem, Score, Solution, Variable};
use utoipa::OpenApi;

use crate::{
    api::controllers::{health_controller, recipe_controller, solve_controller},
    dto::{
        health_response::HealthResponse, recipe_field_kind::RecipeFieldKind,
        recipe_field_response::RecipeFieldResponse,
        recipe_metadata_response::RecipeMetadataResponse, solve_response::SolveResponse,
    },
};

#[derive(OpenApi)]
#[openapi(
    paths(
        health_controller::health,
        recipe_controller::list,
        recipe_controller::metadata,
        recipe_controller::example,
        solve_controller::solve_problem
    ),
    components(schemas(
        HealthResponse,
        RecipeMetadataResponse,
        RecipeFieldResponse,
        RecipeFieldKind,
        SolveResponse,
        Problem,
        Variable,
        Constraint,
        Solution,
        Score
    ))
)]
pub struct ApiDoc;
