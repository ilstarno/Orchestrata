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
