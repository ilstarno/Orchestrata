use axum::{http::StatusCode, response::{IntoResponse, Response}};

use crate::enums::solve_service_error::SolveServiceError;

pub struct ApiError(pub SolveServiceError);

impl From<SolveServiceError> for ApiError {
    fn from(error: SolveServiceError) -> Self {
        Self(error)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
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
