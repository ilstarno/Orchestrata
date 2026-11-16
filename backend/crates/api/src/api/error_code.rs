use axum::{
    http::{HeaderName, HeaderValue},
    response::Response,
};

use serde::Serialize;
use utoipa::ToSchema;

use crate::dto::error_catalog_entry::ErrorCatalogEntry;

pub(super) const ERROR_CODE_HEADER: HeaderName =
    HeaderName::from_static("x-orchestrata-error-code");

#[derive(Clone, Copy, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub(super) enum SolveErrorCode {
    InvalidRequestBody,
    InvalidProblem,
    SearchLimitExceeded,
    SolveExecutionFailed,
    SolverBusy,
}

impl SolveErrorCode {
    pub(super) const ALL: [Self; 5] = [
        Self::InvalidRequestBody,
        Self::InvalidProblem,
        Self::SearchLimitExceeded,
        Self::SolveExecutionFailed,
        Self::SolverBusy,
    ];

    pub(super) const fn entry(self) -> ErrorCatalogEntry {
        match self {
            Self::InvalidRequestBody => ErrorCatalogEntry::new(
                "invalid_request_body",
                "The request body could not be read as a planning problem. Check its content type, structure, and size.",
                &[400, 413, 415, 422],
            ),
            Self::InvalidProblem => ErrorCatalogEntry::new(
                "invalid_problem",
                "The problem was read but failed input validation.",
                &[422],
            ),
            Self::SearchLimitExceeded => ErrorCatalogEntry::new(
                "search_limit_exceeded",
                "The search space exceeds the evaluation limit. No conclusion about feasibility was reached.",
                &[422],
            ),
            Self::SolveExecutionFailed => ErrorCatalogEntry::new(
                "solve_execution_failed",
                "The server could not finish the solve. No conclusion about feasibility was reached.",
                &[500],
            ),
            Self::SolverBusy => ErrorCatalogEntry::new(
                "solver_busy",
                "All solve slots are occupied. Retry later with backoff.",
                &[503],
            ),
        }
    }

    pub(super) fn attach(self, mut response: Response) -> Response {
        response.headers_mut().insert(
            ERROR_CODE_HEADER,
            HeaderValue::from_static(self.entry().code),
        );
        response
    }
}
