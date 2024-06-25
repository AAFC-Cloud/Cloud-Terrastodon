#![feature(let_chains)]
use anyhow::Result;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    info!("Ahoy!");
    entrypoint::prelude::main().await?;
    Ok(())
}
