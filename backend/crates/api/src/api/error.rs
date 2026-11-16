use axum::{
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use orchestrata_application::solve_error::SolveError;

use super::error_code::SolveErrorCode;

pub enum ApiError {
    UnknownRecipe,
    InvalidRequestBody(JsonRejection),
    Solve(SolveError),
    SolveBusy,
}

impl From<SolveError> for ApiError {
    fn from(error: SolveError) -> Self {
        Self::Solve(error)
    }
}

impl From<JsonRejection> for ApiError {
    fn from(error: JsonRejection) -> Self {
        Self::InvalidRequestBody(error)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message, code) = match self {
            Self::UnknownRecipe => {
                return (StatusCode::NOT_FOUND, "Unknown built-in recipe").into_response();
            }
            Self::InvalidRequestBody(rejection) => {
                return SolveErrorCode::InvalidRequestBody.attach(rejection.into_response());
            }
            Self::SolveBusy => (
                StatusCode::SERVICE_UNAVAILABLE,
                "Solver is busy; retry later",
                SolveErrorCode::SolverBusy,
            ),
            Self::Solve(SolveError::InvalidInput) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "Invalid problem",
                SolveErrorCode::InvalidProblem,
            ),
            Self::Solve(SolveError::SearchLimitExceeded) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "Search space exceeds the evaluation limit",
                SolveErrorCode::SearchLimitExceeded,
            ),
            Self::Solve(SolveError::ExecutionFailed) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Solver execution failed",
                SolveErrorCode::SolveExecutionFailed,
            ),
        };
        code.attach((status, message).into_response())
    }
}

#[cfg(test)]
mod tests {
    use axum::body::to_bytes;

    use super::*;

    #[tokio::test]
    async fn solve_errors_keep_existing_status_and_body_with_a_machine_code() {
        for (error, status, message, code) in [
            (
                ApiError::Solve(SolveError::InvalidInput),
                StatusCode::UNPROCESSABLE_ENTITY,
                "Invalid problem",
                "invalid_problem",
            ),
            (
                ApiError::Solve(SolveError::SearchLimitExceeded),
                StatusCode::UNPROCESSABLE_ENTITY,
                "Search space exceeds the evaluation limit",
                "search_limit_exceeded",
            ),
            (
                ApiError::Solve(SolveError::ExecutionFailed),
                StatusCode::INTERNAL_SERVER_ERROR,
                "Solver execution failed",
                "solve_execution_failed",
            ),
            (
                ApiError::SolveBusy,
                StatusCode::SERVICE_UNAVAILABLE,
                "Solver is busy; retry later",
                "solver_busy",
            ),
        ] {
            let response = error.into_response();
            assert_eq!(response.status(), status);
            assert_eq!(response.headers()["x-orchestrata-error-code"], code);
            assert_eq!(to_bytes(response.into_body(), 1024).await.unwrap(), message);
        }
    }
}
