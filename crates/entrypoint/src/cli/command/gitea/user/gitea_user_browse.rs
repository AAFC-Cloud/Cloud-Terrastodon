use clap::Args;
use cloud_terrastodon_command::CacheInvalidatableIntoFuture;
use cloud_terrastodon_gitea::GiteaTenantArgument;
use cloud_terrastodon_gitea::GiteaTenantArgumentExt;
use cloud_terrastodon_gitea::fetch_all_gitea_users;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use std::io::Write;

#[derive(Args, Debug, Clone)]
pub struct GiteaUserBrowseArgs {
    /// Tracked tenant URL or alias to query. Defaults to the active `tea` login.
    #[arg(long, default_value_t)]
    pub tenant: GiteaTenantArgument<'static>,
}

impl GiteaUserBrowseArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenant = self.tenant.resolve().await?;
        let chosen = PickerTui::new()
            .pick_many_reloadable(async |invalidate| {
                fetch_all_gitea_users(&tenant)
                    .with_invalidation(invalidate)
                    .await
            })
            .await?;
        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        serde_json::to_writer_pretty(&mut handle, &chosen)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
