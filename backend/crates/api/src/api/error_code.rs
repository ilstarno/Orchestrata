use axum::{
    http::{HeaderName, HeaderValue},
    response::Response,
};

use serde::Serialize;
use utoipa::ToSchema;

const ERROR_CODE_HEADER: HeaderName = HeaderName::from_static("x-orchestrata-error-code");

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub(super) enum SolveErrorCode {
    InvalidRequestBody,
    InvalidProblem,
    SearchLimitExceeded,
    SolveExecutionFailed,
    SolverBusy,
}

impl SolveErrorCode {
    fn as_str(&self) -> &'static str {
        match self {
            Self::InvalidRequestBody => "invalid_request_body",
            Self::InvalidProblem => "invalid_problem",
            Self::SearchLimitExceeded => "search_limit_exceeded",
            Self::SolveExecutionFailed => "solve_execution_failed",
            Self::SolverBusy => "solver_busy",
        }
    }

    pub(super) fn attach(self, mut response: Response) -> Response {
        response
            .headers_mut()
            .insert(ERROR_CODE_HEADER, HeaderValue::from_static(self.as_str()));
        response
    }
}
