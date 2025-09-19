use cloud_terrastodon_pathing::AppDir;
use cloud_terrastodon_pathing::Existy;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerTui;
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
            format!("{dir} (does not exist yet)")
        };
        choices.push(Choice {
            key: display,
            value: (dir, exists),
        });
    }
    let dirs_to_open = PickerTui::new(choices)
        .set_header("Choose directories to open")
        .pick_many()?;
    for v in dirs_to_open {
    let (dir, exists) = v;
        if !exists {
            dir.as_path_buf().ensure_dir_exists().await?;
        }
        open(dir.as_path_buf())?;
    }

    Ok(())
}
