use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use orchestrata_application::solve_error::SolveError;

pub enum ApiError {
    Solve(SolveError),
    SolveBusy,
}

impl From<SolveError> for ApiError {
    fn from(error: SolveError) -> Self {
        Self::Solve(error)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::SolveBusy => (
                StatusCode::SERVICE_UNAVAILABLE,
                "Solver is busy; retry later",
            ),
            Self::Solve(SolveError::InvalidInput) => {
                (StatusCode::UNPROCESSABLE_ENTITY, "Invalid problem")
            }
            Self::Solve(SolveError::SearchLimitExceeded) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "Search space exceeds the evaluation limit",
            ),
            Self::Solve(SolveError::ExecutionFailed) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Solver execution failed")
            }
        };
        (status, message).into_response()
    }
}
