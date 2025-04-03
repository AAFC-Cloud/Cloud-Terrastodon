#![feature(let_chains)]
use cloud_terrastodon_core_entrypoint::prelude::Version;
use cloud_terrastodon_core_entrypoint::prelude::entrypoint;
use eyre::Result;
#[tokio::main]
async fn main() -> Result<()> {
    let version = Version::new(env!("CARGO_PKG_VERSION").to_string());
    entrypoint(version).await?;
    Ok(())
}
