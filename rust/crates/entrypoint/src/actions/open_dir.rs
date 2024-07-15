use anyhow::Result;
use fzf::pick_many;
use fzf::Choice;
use fzf::FzfArgs;
use opener::open;
use pathing::AppDir;
use pathing::Existy;
use tokio::fs::try_exists;

pub async fn open_dir() -> Result<()> {
    let mut choices = Vec::new();
    for dir in AppDir::variants() {
        let exists = try_exists(dir.as_path_buf()).await?;
        let display = if exists {
            dir.to_string()
        } else {
            format!("{} (does not exist yet)", dir)
        };
        choices.push(Choice {
            display,
            inner: (dir, exists),
        });
    }
    let dirs_to_open = pick_many(FzfArgs {
        choices,
        prompt: None,
        header: Some("Choose directories to open".to_string()),
    })?;
    for v in dirs_to_open {
        let (dir, exists) = v.inner;
        if !exists {
            dir.as_path_buf().ensure_dir_exists().await?;
        }
        open(dir.as_path_buf())?;
    }

    Ok(())
}
