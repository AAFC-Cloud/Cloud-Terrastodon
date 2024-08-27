use anyhow::Result;
use cloud_terrasotodon_core_pathing::AppDir;
use std::path::PathBuf;
use tokio::fs;
use tracing::info;
pub async fn clean_processed() -> Result<()> {
    info!("Cleaning processed");
    let ignore_dir: PathBuf = AppDir::Processed.into();
    fs::remove_dir_all(ignore_dir).await?;
    Ok(())
}
