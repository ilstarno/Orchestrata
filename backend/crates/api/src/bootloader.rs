use axum::Router;

use crate::api;

pub struct Bootloader;

impl Bootloader {
    pub async fn start() -> Result<(), Box<dyn std::error::Error>> {
        let app = Self::build();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;

        axum::serve(listener, app).await?;
        Ok(())
    }

    pub fn build() -> Router {
        Router::new().merge(api::router())
    }
}
