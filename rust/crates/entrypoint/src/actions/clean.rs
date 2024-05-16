use anyhow::Result;
use std::path::PathBuf;
use tokio::fs;
use tracing::info;
pub async fn clean() -> Result<()> {
    info!("Cleaning");
    let ignore_dir = PathBuf::from("ignore");
    fs::remove_dir_all(ignore_dir).await?;
    Ok(())
}
