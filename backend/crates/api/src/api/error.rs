use axum::{
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use orchestrata_application::solve_error::SolveError;

use super::error_code::SolveErrorCode;

pub enum ApiError {
    UnknownRecipe,
    InvalidRecipeGridQuery,
    RecipeCatalogUnavailable,
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
            Self::InvalidRecipeGridQuery => {
                return (StatusCode::BAD_REQUEST, "Invalid recipe grid query").into_response();
            }
            Self::RecipeCatalogUnavailable => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Built-in recipe metadata is unavailable",
                )
                    .into_response();
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
#[path = "../../../../../backend_tests/api/error_unit.rs"]
mod tests;
