#![cfg(feature = "entrypoint")]
use cloud_terrastodon_entrypoint::GitRevision;
use cloud_terrastodon_entrypoint::Version;
use cloud_terrastodon_entrypoint::entrypoint;

fn main() -> eyre::Result<()> {
    let version = Version::new(env!("CARGO_PKG_VERSION").to_string());
    let revision = GitRevision::new(option_env!("GIT_REVISION").unwrap_or("unknown").to_string());
    entrypoint(version, revision)?;
    Ok(())
}
