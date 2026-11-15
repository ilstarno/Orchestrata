#[path = "api/module.rs"]
mod api;
mod bootloader;
#[path = "config/module.rs"]
mod config;
#[path = "dto/module.rs"]
mod dto;

use std::process::ExitCode;

use bootloader::Bootloader;

#[tokio::main]
async fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .json()
        .with_writer(std::io::stderr)
        .with_target(false)
        .with_max_level(tracing::Level::INFO)
        .with_current_span(true)
        .with_span_list(false)
        .init();
    match Bootloader::start().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            tracing::error!(event = "api_start_failed", %error);
            ExitCode::FAILURE
        }
    }
}
