use axum::http::StatusCode;

use crate::solve_service_error::SolveServiceError;

pub struct ApiError(pub SolveServiceError);

impl From<SolveServiceError> for ApiError {
    fn from(error: SolveServiceError) -> Self {
        Self(error)
    }
}

impl axum::response::IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self.0 {
            SolveServiceError::InvalidProblem => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "Invalid problem or search space too large",
            ),
            SolveServiceError::ExecutionFailed => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Solver execution failed",
            ),
        };

        (status, message).into_response()
    }
}
