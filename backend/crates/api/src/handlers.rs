use axum::{Json, http::StatusCode};
use orchestrata_solver::{Problem, solve};

use crate::{health_response::HealthResponse, solve_response::SolveResponse};

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
        (status = 422, description = "Invalid problem or search space too large")
    )
)]
pub async fn solve_problem(
    Json(problem): Json<Problem>,
) -> Result<Json<SolveResponse>, (StatusCode, &'static str)> {
    let solution = tokio::task::spawn_blocking(move || solve(&problem, 100_000))
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Solver worker failed"))?
        .map_err(|_| (StatusCode::UNPROCESSABLE_ENTITY, "Invalid problem or search space too large"))?;

    Ok(Json(SolveResponse { solution }))
}
