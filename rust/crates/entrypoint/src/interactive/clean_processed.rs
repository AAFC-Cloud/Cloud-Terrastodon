use cloud_terrastodon_core_pathing::AppDir;
use eyre::Result;
use std::path::PathBuf;
use tokio::fs;
use tracing::info;
pub async fn clean_processed() -> Result<()> {
    info!("Cleaning processed");
    let processed_dir: PathBuf = AppDir::Processed.into();
    fs::remove_dir_all(processed_dir).await?;
    Ok(())
}
