use orchestrata_solver::{Constraint, Problem, Score, Solution, Variable};
use utoipa::OpenApi;

use crate::{
    api::{
        controllers::{health_controller, solve_controller},
        error_code::SolveErrorCode,
    },
    dto::{health_response::HealthResponse, solve_response::SolveResponse},
};

#[derive(OpenApi)]
#[openapi(
    paths(health_controller::health, solve_controller::solve_problem),
    components(schemas(
        HealthResponse,
        SolveResponse,
        SolveErrorCode,
        Problem,
        Variable,
        Constraint,
        Solution,
        Score
    ))
)]
pub struct ApiDoc;
