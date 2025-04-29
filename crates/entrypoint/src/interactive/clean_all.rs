use cloud_terrastodon_pathing::AppDir;
use cloud_terrastodon_user_input::FzfArgs;
use cloud_terrastodon_user_input::pick;
use eyre::Result;
use eyre::bail;
use tokio::fs;
use tracing::info;
pub async fn clean_all_menu() -> Result<()> {
    let choices = vec!["keep command cache (recommended)", "purge command cache"];
    let chosen = pick(FzfArgs {
        choices,

        header: Some("Cleaning".to_string()),
        ..Default::default()
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
