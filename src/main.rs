use cloud_terrastodon_entrypoint::prelude::Version;
use cloud_terrastodon_entrypoint::prelude::entrypoint;

fn main() -> eyre::Result<()> {
    let version = Version::new(env!("CARGO_PKG_VERSION").to_string());
    entrypoint(version)?;
    Ok(())
}
