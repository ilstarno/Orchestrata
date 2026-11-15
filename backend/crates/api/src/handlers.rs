use axum::Json;
use orchestrata_solver::Problem;

use crate::{
    api_error::ApiError,
    health_response::HealthResponse,
    solve_response::SolveResponse,
    solve_service::SolveService,
};

#[utoipa::path(
    get,
    path = "/api/v1/health",
    responses((status = 200, body = HealthResponse))
)]
pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

#[utoipa::path(
    post,
    path = "/api/v1/solve",
    request_body = Problem,
    responses(
        (status = 200, body = SolveResponse),
        (status = 422, description = "Invalid problem or search space too large"),
        (status = 500, description = "Solver execution failed")
    )
)]
pub async fn solve_problem(
    Json(problem): Json<Problem>,
) -> Result<Json<SolveResponse>, ApiError> {
    let solution = SolveService::execute(problem).await?;
    Ok(Json(SolveResponse { solution }))
}
