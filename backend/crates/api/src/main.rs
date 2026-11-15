#[path = "api/module.rs"]
mod api;
mod bootloader;
#[path = "dto/module.rs"]
mod dto;

use bootloader::Bootloader;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    Bootloader::start().await
}
