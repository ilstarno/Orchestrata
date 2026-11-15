mod access_log;
#[cfg(test)]
mod access_log_tests;
#[path = "controllers/module.rs"]
pub mod controllers;
pub mod error;
#[cfg(test)]
mod error_catalog_tests;
mod error_code;
mod http_error_code;
pub mod openapi;
mod solve_admission;

use axum::{
    Json, Router, middleware,
    routing::{get, post},
};
use utoipa::OpenApi;

use crate::api::{
    controllers::{error_catalog_controller, health_controller, solve_controller},
    openapi::ApiDoc,
};

pub fn router() -> Router {
    let max_concurrent = std::thread::available_parallelism().map_or(1, usize::from);
    let admission = solve_admission::SolveAdmission::new(max_concurrent);
    Router::new()
        .route("/api/v1/health", get(health_controller::health))
        .route(
            "/api/v1/error-codes",
            get(error_catalog_controller::error_catalog),
        )
        .route("/api/v1/solve", post(solve_controller::solve_problem))
        .route(
            "/api/v1/openapi.json",
            get(|| async { Json(ApiDoc::openapi()) }),
        )
        .layer(middleware::from_fn(
            http_error_code::attach_unclassified_code,
        ))
        .layer(middleware::from_fn(access_log::observe_request))
        .with_state(admission)
}
