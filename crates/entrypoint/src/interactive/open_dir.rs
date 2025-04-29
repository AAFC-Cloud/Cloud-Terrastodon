use cloud_terrastodon_pathing::AppDir;
use cloud_terrastodon_pathing::Existy;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::FzfArgs;
use cloud_terrastodon_user_input::pick_many;
use eyre::Result;
use opener::open;
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
            key: display,
            value: (dir, exists),
        });
    }
    let dirs_to_open = pick_many(FzfArgs {
        choices,

        header: Some("Choose directories to open".to_string()),
        ..Default::default()
    })?;
    for v in dirs_to_open {
        let (dir, exists) = v.value;
        if !exists {
            dir.as_path_buf().ensure_dir_exists().await?;
        }
        open(dir.as_path_buf())?;
    }

    Ok(())
}
