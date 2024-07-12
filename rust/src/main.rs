#![feature(let_chains)]
use anyhow::Result;
use entrypoint::prelude::Version;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    info!("Ahoy!");
    entrypoint::prelude::main(Version::new(env!("CARGO_PKG_VERSION").to_string())).await?;
    Ok(())
}
