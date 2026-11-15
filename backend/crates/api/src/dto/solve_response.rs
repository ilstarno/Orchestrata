use orchestrata_solver::Solution;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct SolveResponse {
    pub solution: Option<Solution>,
}
