use axum::{Json, extract::State};
use orchestrata_solver::Problem;

use crate::{
    api::{error::ApiError, solve_admission::SolveAdmission},
    dto::solve_response::SolveResponse,
};

#[utoipa::path(
    post,
    path = "/api/v1/solve",
    request_body = Problem,
    responses(
        (status = 200, description = "Optimal solution; solution field is null if proven infeasible", body = SolveResponse),
        (status = 422, description = "Invalid problem or search space too large"),
        (status = 500, description = "Solver execution failed"),
        (status = 503, description = "All solve execution slots are occupied; retry later")
    )
)]
pub async fn solve_problem(
    State(admission): State<SolveAdmission>,
    Json(problem): Json<Problem>,
) -> Result<Json<SolveResponse>, ApiError> {
    let solution = admission.solve(problem).await?;
    Ok(Json(SolveResponse { solution }))
}
