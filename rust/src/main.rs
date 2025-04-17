#![feature(let_chains)]
use cloud_terrastodon_core_entrypoint::prelude::Version;
use cloud_terrastodon_core_entrypoint::prelude::entrypoint;
use eyre::Result;
fn main() -> Result<()> {
    let version = Version::new(env!("CARGO_PKG_VERSION").to_string());
    entrypoint(version)?;
    Ok(())
}
