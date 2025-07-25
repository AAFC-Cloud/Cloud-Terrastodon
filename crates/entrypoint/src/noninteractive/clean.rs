use cloud_terrastodon_pathing::AppDir;
use eyre::Result;
use tokio::fs::remove_dir_all;
use tracing::info;
use tracing::warn;

pub async fn clean() -> Result<()> {
    for dir in AppDir::ok_to_clean() {
        info!("Cleaning {dir}...");
        if !dir.as_path_buf().exists() {
            info!("Directory {dir} does not exist, skipping.");
            continue;
        }
        if let Err(e) = remove_dir_all(dir.as_path_buf()).await {
            warn!("Ignoring error encountered cleaning {dir}: {e:?}");
        }
    }
    Ok(())
}
