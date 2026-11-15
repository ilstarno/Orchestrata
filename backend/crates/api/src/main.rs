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
    match Bootloader::start().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("API startup or server error: {error}");
            ExitCode::FAILURE
        }
    }
}
