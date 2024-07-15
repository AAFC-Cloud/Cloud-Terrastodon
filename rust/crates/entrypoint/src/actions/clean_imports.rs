use anyhow::Result;
use pathing::AppDir;
use std::path::PathBuf;
use tokio::fs;
use tracing::info;
pub async fn clean_imports() -> Result<()> {
    info!("Cleaning imports");
    let ignore_dir: PathBuf = AppDir::Imports.into();
    fs::remove_dir_all(ignore_dir).await?;
    Ok(())
}
