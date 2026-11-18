use axum::{
    Json, Router,
    extract::DefaultBodyLimit,
    middleware,
    routing::{get, post},
};
use utoipa::OpenApi;

use crate::api::{
    access_log,
    controllers::{
        error_catalog_controller, health_controller, lead_preview_controller,
        lead_preview_web_controller, recipe_controller, recipe_grid_controller, solve_controller,
    },
    http_error_code,
    openapi::ApiDoc,
    solve_admission,
};

pub fn router() -> Router {
    let max_concurrent = std::thread::available_parallelism().map_or(1, usize::from);
    router_with_admission(solve_admission::SolveAdmission::new(max_concurrent))
}

pub(super) fn router_with_admission(admission: solve_admission::SolveAdmission) -> Router {
    Router::new()
        .route("/api/v1/health", get(health_controller::health))
        .route(
            "/examples/lead-to-contact",
            get(lead_preview_web_controller::catalog),
        )
        .route(
            "/examples/lead-to-contact/{lead_id}",
            get(lead_preview_web_controller::view),
        )
        .route(
            "/api/v1/examples/lead-to-contact",
            get(lead_preview_controller::catalog),
        )
        .route(
            "/api/v1/examples/lead-to-contact/{lead_id}/preview",
            post(lead_preview_controller::evaluate).layer(DefaultBodyLimit::max(16)),
        )
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
