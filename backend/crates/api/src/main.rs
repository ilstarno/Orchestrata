mod handlers;
mod health_response;
mod openapi;
mod solve_response;

use axum::{Json, Router, routing::{get, post}};
use utoipa::OpenApi;

use crate::openapi::ApiDoc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new()
        .route("/api/v1/health", get(handlers::health))
        .route("/api/v1/solve", post(handlers::solve_problem))
        .route("/api/v1/openapi.json", get(|| async { Json(ApiDoc::openapi()) }));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
