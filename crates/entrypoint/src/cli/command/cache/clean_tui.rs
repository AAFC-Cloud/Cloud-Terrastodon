use clap::Args;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::discover_caches;
use cloud_terrastodon_pathing::AppDir;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;

#[derive(Args, Debug, Clone)]
pub struct CacheCleanTuiArgs {}

impl CacheCleanTuiArgs {
    pub async fn invoke(self) -> Result<()> {
        // Discover caches under AppDir::Commands
        let root = AppDir::Commands.as_path_buf();
        let caches = discover_caches(&root).await?;

        // Build choices for the picker: show the disk path and last-used timestamp
        let choices: Vec<Choice<CacheKey>> = caches
            .into_iter()
            .map(|(cache_key, dt)| {
                Choice {
                    key: format!("{} ({})", cache_key.path.display(), dt.to_rfc2822()),
                    value: cache_key,
                }
            })
            .collect();

        // Let the user pick multiple cache entries to invalidate
        let picked = PickerTui::new()
            .set_header("Select cache entries to invalidate")
            .pick_many(choices)?;

        for key in picked {
            key.invalidate().await?;
            println!("Invalidated cache: {}", key.path_on_disk().display());
        }

        Ok(())
    }
}
