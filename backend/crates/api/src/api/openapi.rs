use orchestrata_solver::{Constraint, Problem, Score, Solution, Variable};
use utoipa::OpenApi;

use crate::{
    api::{
        controllers::{error_catalog_controller, health_controller, solve_controller},
        error_code::SolveErrorCode,
    },
    dto::{
        error_catalog_entry::ErrorCatalogEntry, health_response::HealthResponse,
        solve_response::SolveResponse,
    },
};

#[derive(OpenApi)]
#[openapi(
    paths(
        error_catalog_controller::error_catalog,
        health_controller::health,
        solve_controller::solve_problem
    ),
    components(schemas(
        HealthResponse,
        SolveResponse,
        SolveErrorCode,
        ErrorCatalogEntry,
        Problem,
        Variable,
        Constraint,
        Solution,
        Score
    ))
)]
pub struct ApiDoc;
