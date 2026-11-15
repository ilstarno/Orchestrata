use orchestrata_solver::{Constraint, Problem, Score, Solution, Variable};
use utoipa::OpenApi;

use crate::{handlers, health_response::HealthResponse, solve_response::SolveResponse};

#[derive(OpenApi)]
#[openapi(
    paths(handlers::health, handlers::solve_problem),
    components(schemas(
        HealthResponse, SolveResponse, Problem, Variable, Constraint, Solution, Score
    )),
    tags((name = "orchestrata", description = "Optimization API"))
)]
pub struct ApiDoc;
