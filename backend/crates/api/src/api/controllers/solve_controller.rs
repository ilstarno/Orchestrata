use axum::Json;
use orchestrata_solver::Problem;

use crate::{
    api::error::ApiError,
    dto::solve_response::SolveResponse,
};
use orchestrata_application::solve_service::SolveService;

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
    let solution = SolveService::execute_async(problem).await?;
    Ok(Json(SolveResponse { solution }))
}
