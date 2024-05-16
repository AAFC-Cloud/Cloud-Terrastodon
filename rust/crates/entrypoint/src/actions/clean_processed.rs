use anyhow::Result;
use tokio::fs;
use std::path::PathBuf;
use tracing::info;
pub async fn clean_processed() -> Result<()> {
    info!("Cleaning processed");
    let ignore_dir = PathBuf::from_iter(["ignore","processed"]);
    fs::remove_dir_all(ignore_dir).await?;
    Ok(())
}
