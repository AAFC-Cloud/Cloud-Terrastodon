use anyhow::bail;
use anyhow::Result;
use cloud_terrastodon_core_fzf::pick;
use cloud_terrastodon_core_fzf::FzfArgs;
use cloud_terrastodon_core_pathing::AppDir;
use tokio::fs;
use tracing::info;
pub async fn clean_all_menu() -> Result<()> {
    let choices = vec!["keep command cache (recommended)", "purge command cache"];
    let chosen = pick(FzfArgs {
        choices,
        prompt: None,
        header: Some("Cleaning".to_string()),
    })?;
    match chosen {
        "keep command cache (recommended)" => {
            info!("Cleaning everything except the command cache");
            for dir in [AppDir::Imports, AppDir::Processed] {
                // ignore errors if not exists
                let _ = fs::remove_dir_all(dir.as_path_buf()).await;
            }
            Ok(())
        }
        "purge command cache" => {
            info!("Cleaning everything including command cache");
            for dir in AppDir::ok_to_clean() {
                // ignore errors if not exists
                let _ = fs::remove_dir_all(dir.as_path_buf()).await;
            }
            Ok(())
        }
        bad => {
            bail!("Invalid clean choice: {bad}");
        }
    }
}
