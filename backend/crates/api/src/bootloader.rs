use axum::Router;

use crate::{api, config::RuntimeConfig};

pub struct Bootloader;

impl Bootloader {
    pub async fn start() -> Result<(), Box<dyn std::error::Error>> {
        let app = Self::build();
        let config = RuntimeConfig::from_environment()?;
        let listener = tokio::net::TcpListener::bind(config.http_bind)
            .await
            .map_err(|source| {
                std::io::Error::new(
                    source.kind(),
                    format!(
                        "could not bind HTTP listener to {}: {source}",
                        config.http_bind
                    ),
                )
            })?;

        axum::serve(listener, app).await?;
        Ok(())
    }

    pub fn build() -> Router {
        Router::new().merge(api::router())
    }
}
