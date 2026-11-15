#[path = "controllers/module.rs"]
pub mod controllers;
pub mod error;
mod error_code;
pub mod openapi;
mod solve_admission;

use axum::{
    Json, Router,
    routing::{get, post},
};
use utoipa::OpenApi;

use crate::api::{
    controllers::{health_controller, solve_controller},
    openapi::ApiDoc,
};

pub fn router() -> Router {
    let max_concurrent = std::thread::available_parallelism().map_or(1, usize::from);
    let admission = solve_admission::SolveAdmission::new(max_concurrent);
    Router::new()
        .route("/api/v1/health", get(health_controller::health))
        .route("/api/v1/solve", post(solve_controller::solve_problem))
        .route(
            "/api/v1/openapi.json",
            get(|| async { Json(ApiDoc::openapi()) }),
        )
        .with_state(admission)
}
