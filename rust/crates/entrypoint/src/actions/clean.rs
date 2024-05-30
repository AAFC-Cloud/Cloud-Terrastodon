use anyhow::Result;
use pathing_types::IgnoreDir;
use std::path::PathBuf;
use tokio::fs;
use tracing::info;
pub async fn clean() -> Result<()> {
    info!("Cleaning");
    let ignore_dir: PathBuf = IgnoreDir::Root.into();
    fs::remove_dir_all(ignore_dir).await?;
    Ok(())
}
