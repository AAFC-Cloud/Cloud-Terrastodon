use anyhow::Result;
use cloud_terrasotodon_core_pathing::AppDir;
use tokio::fs::remove_dir_all;
use tracing::info;
use tracing::warn;

pub async fn clean() -> Result<()> {
    for dir in AppDir::ok_to_clean() {
        info!("Cleaning {dir}...");
        if let Err(e) = remove_dir_all(dir.as_path_buf()).await {
            warn!("Ignoring error encountered cleaning {dir}: {e:?}");
        }
    }
    Ok(())
}
