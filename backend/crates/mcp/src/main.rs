use std::error::Error;

use rmcp::{ServiceExt, transport::stdio};

mod recipe_id;
mod recipe_server;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    recipe_server::RecipeServer
        .serve(stdio())
        .await?
        .waiting()
        .await?;
    Ok(())
}
