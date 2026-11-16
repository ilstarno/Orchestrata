mod access_log;
#[cfg(test)]
mod access_log_tests;
#[path = "controllers/module.rs"]
pub mod controllers;
pub mod error;
#[cfg(test)]
mod error_catalog_tests;
mod error_code;
#[cfg(test)]
mod fleet_examples_tests;
mod http_error_code;
pub mod openapi;
mod recipe_catalog;
#[cfg(test)]
mod recipe_catalog_tests;
mod recipe_grid;
#[cfg(test)]
#[path = "../../../../../backend_tests/api/recipe_grid.rs"]
mod recipe_grid_tests;
#[cfg(test)]
mod recipe_run_tests;
mod solve_admission;

use axum::{
    Json, Router, middleware,
    routing::{get, post},
};
use utoipa::OpenApi;

use crate::api::{
    controllers::{
        error_catalog_controller, health_controller, recipe_controller, recipe_grid_controller,
        solve_controller,
    },
    openapi::ApiDoc,
};

pub fn router() -> Router {
    let max_concurrent = std::thread::available_parallelism().map_or(1, usize::from);
    router_with_admission(solve_admission::SolveAdmission::new(max_concurrent))
}

fn router_with_admission(admission: solve_admission::SolveAdmission) -> Router {
    Router::new()
        .route("/api/v1/health", get(health_controller::health))
        .route(
            "/api/v1/error-codes",
            get(error_catalog_controller::error_catalog),
        )
        .route("/api/v1/solve", post(solve_controller::solve_problem))
        .route("/api/v1/recipes", get(recipe_controller::list))
        .route("/api/v1/recipes/grid", get(recipe_grid_controller::grid))
        .route(
            "/api/v1/recipes/{recipe_id}",
            get(recipe_controller::metadata),
        )
        .route(
            "/api/v1/recipes/{recipe_id}/example",
            get(recipe_controller::example),
        )
        .route(
            "/api/v1/recipes/{recipe_id}/run-example",
            post(recipe_controller::run_example),
        )
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
