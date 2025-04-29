use cloud_terrastodon_core_pathing::AppDir;
use eyre::Result;
use std::path::PathBuf;
use tokio::fs;
use tracing::info;
pub async fn clean_imports() -> Result<()> {
    info!("Cleaning imports");
    let imports_dir: PathBuf = AppDir::Imports.into();
    fs::remove_dir_all(imports_dir).await?;
    Ok(())
}
