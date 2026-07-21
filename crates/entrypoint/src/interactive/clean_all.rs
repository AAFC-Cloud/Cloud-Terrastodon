use crate::noninteractive::clean_dirs;
use cloud_terrastodon_pathing::AppDir;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use eyre::bail;
pub async fn clean_all_menu() -> Result<()> {
    let choices = vec!["keep command cache (recommended)", "purge command cache"];
    let chosen = PickerTui::<_>::new()
        .set_header("Cleaning")
        .pick_one(choices)
        .await?;
    match chosen {
        "keep command cache (recommended)" => {
            clean_dirs([AppDir::Imports, AppDir::Processed]).await?;
            Ok(())
        }
        "purge command cache" => {
            clean_dirs(AppDir::ok_to_clean()).await?;
            Ok(())
        }
        bad => {
            bail!("Invalid clean choice: {bad}");
        }
    }
}
