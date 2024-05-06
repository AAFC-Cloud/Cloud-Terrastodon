use anyhow::Result;
use tokio::fs;
use std::path::PathBuf;
use tracing::info;
pub async fn clean_imports() -> Result<()> {
    info!("Cleaning imports");
    let ignore_dir = PathBuf::from_iter(["ignore","imports"]);
    fs::remove_dir_all(ignore_dir).await?;
    Ok(())
}
