use anyhow::Result;
use pathing_types::IgnoreDir;
use std::path::PathBuf;
use tokio::fs;
use tracing::info;
pub async fn clean_processed() -> Result<()> {
    info!("Cleaning processed");
    let ignore_dir: PathBuf = IgnoreDir::Processed.into();
    fs::remove_dir_all(ignore_dir).await?;
    Ok(())
}
