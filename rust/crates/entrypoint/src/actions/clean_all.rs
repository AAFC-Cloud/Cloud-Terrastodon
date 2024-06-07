use anyhow::bail;
use anyhow::Result;
use fzf::FzfArgs;
use pathing_types::IgnoreDir;
use std::path::PathBuf;
use tokio::fs;
use tracing::info;
pub async fn clean_all() -> Result<()> {
    let choices = vec!["keep command cache (recommended)", "purge command cache"];
    let chosen = fzf::pick(FzfArgs {
        choices,
        prompt: None,
        header: Some("Cleaning".to_string()),
    })?;
    match chosen {
        "keep command cache (recommended)" => {
            info!("Cleaning everything except the command cache");
            for dir in [IgnoreDir::Imports, IgnoreDir::Processed] {
                // ignore errors if not exists
                let _ = fs::remove_dir_all(dir.as_path_buf()).await;
            }
            Ok(())
        }
        "purge command cache" => {
            info!("Cleaning everything including command cache");
            let ignore_dir: PathBuf = IgnoreDir::Root.into();
            fs::remove_dir_all(ignore_dir).await?;
            Ok(())
        }
        bad => {
            bail!("Invalid clean choice: {bad}");
        }
    }
}
