mod api;
mod constants;
pub mod domain;
mod dto;
pub mod engine;
mod enums;
mod services;

use axum::{Json, Router, routing::{get, post}};
use utoipa::OpenApi;

use crate::api::{
    controllers::{health_controller, solve_controller},
    openapi::ApiDoc,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new()
        .route("/api/v1/health", get(health_controller::health))
        .route("/api/v1/solve", post(solve_controller::solve_problem))
        .route("/api/v1/openapi.json", get(|| async { Json(ApiDoc::openapi()) }));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
