use anyhow::Result;
use pathing_types::IgnoreDir;
use std::path::PathBuf;
use tokio::fs;
use tracing::info;
pub async fn clean_imports() -> Result<()> {
    info!("Cleaning imports");
    let ignore_dir: PathBuf = IgnoreDir::Imports.into();
    fs::remove_dir_all(ignore_dir).await?;
    Ok(())
}
