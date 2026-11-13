use crate::noninteractive::clean_dirs;
use cloud_terrastodon_cache::WorkCache;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_pathing::AppDir;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use eyre::bail;
pub async fn clean_all_menu(cache: &WorkCache, cancellation: &CancellationScope) -> Result<()> {
    let choices = vec![
        "keep command cache (recommended)",
        "invalidate command cache",
    ];
    let chosen = PickerTui::<_>::new()
        .set_header("Cleaning")
        .pick_one(choices, cancellation)
        .await?;
    match chosen {
        "keep command cache (recommended)" => {
            clean_dirs([AppDir::Imports, AppDir::Processed], cache, cancellation).await?;
            Ok(())
        }
        "invalidate command cache" => {
            clean_dirs(AppDir::ok_to_clean(), cache, cancellation).await?;
            Ok(())
        }
        bad => {
            bail!("Invalid clean choice: {bad}");
        }
    }
}
