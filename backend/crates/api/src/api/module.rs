#[path = "controllers/module.rs"]
pub mod controllers;
pub mod error;
#[cfg(test)]
mod fleet_examples_tests;
pub mod openapi;
mod recipe_catalog;
#[cfg(test)]
mod recipe_catalog_tests;

use axum::{
    Json, Router,
    routing::{get, post},
};
use utoipa::OpenApi;

use crate::api::{
    controllers::{health_controller, recipe_controller, solve_controller},
    openapi::ApiDoc,
};

pub fn router() -> Router {
    Router::new()
        .route("/api/v1/health", get(health_controller::health))
        .route("/api/v1/solve", post(solve_controller::solve_problem))
        .route("/api/v1/recipes", get(recipe_controller::list))
        .route(
            "/api/v1/recipes/{recipe_id}",
            get(recipe_controller::metadata),
        )
        .route(
            "/api/v1/recipes/{recipe_id}/example",
            get(recipe_controller::example),
        )
        .route(
            "/api/v1/openapi.json",
            get(|| async { Json(ApiDoc::openapi()) }),
        )
}
