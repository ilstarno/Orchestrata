use axum::{http::StatusCode, response::{IntoResponse, Response}};
use orchestrata_application::solve_error::SolveError;

pub struct ApiError(pub SolveError);

impl From<SolveError> for ApiError {
    fn from(error: SolveError) -> Self {
        Self(error)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self.0 {
            SolveError::InvalidInput => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "Invalid problem or search space too large",
            ),
            SolveError::ExecutionFailed => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Solver execution failed",
            ),
        };
        (status, message).into_response()
    }
}
