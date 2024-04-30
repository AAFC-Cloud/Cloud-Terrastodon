#![feature(let_chains)]
use anyhow::Result;
use entrypoint::prelude::menu_loop;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    info!("Ahoy!");
    menu_loop().await?;

    Ok(())
}
