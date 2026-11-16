use axum::{
    Json,
    extract::{State, rejection::JsonRejection},
};
use orchestrata_solver::Problem;

use crate::{
    api::{error::ApiError, error_code::SolveErrorCode, solve_admission::SolveAdmission},
    dto::solve_response::SolveResponse,
};

#[utoipa::path(
    post,
    path = "/api/v1/solve",
    request_body = Problem,
    responses(
        (status = 200, description = "Optimal solution; solution field is null if proven infeasible", body = SolveResponse),
        (status = 400, description = "Malformed or unreadable request body",
            headers(("X-Orchestrata-Error-Code" = SolveErrorCode, description = "invalid_request_body"))),
        (status = 413, description = "Request body exceeds the accepted size",
            headers(("X-Orchestrata-Error-Code" = SolveErrorCode, description = "invalid_request_body"))),
        (status = 415, description = "Expected a JSON content type",
            headers(("X-Orchestrata-Error-Code" = SolveErrorCode, description = "invalid_request_body"))),
        (status = 422, description = "Invalid JSON shape, invalid problem, or search space too large",
            headers(("X-Orchestrata-Error-Code" = SolveErrorCode, description = "invalid_request_body, invalid_problem, or search_limit_exceeded"))),
        (status = 500, description = "Solver execution failed",
            headers(("X-Orchestrata-Error-Code" = SolveErrorCode, description = "solve_execution_failed"))),
        (status = 503, description = "All solve execution slots are occupied; retry later",
            headers(("X-Orchestrata-Error-Code" = SolveErrorCode, description = "solver_busy")))
    )
)]
pub async fn solve_problem(
    State(admission): State<SolveAdmission>,
    body: Result<Json<Problem>, JsonRejection>,
) -> Result<Json<SolveResponse>, ApiError> {
    let Json(problem) = body?;
    let solution = admission.solve(problem).await?;
    Ok(Json(SolveResponse { solution }))
}
