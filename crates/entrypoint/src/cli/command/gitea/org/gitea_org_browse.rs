use clap::Args;
use cloud_terrastodon_command::CacheInvalidatableIntoFuture;
use cloud_terrastodon_gitea::GiteaTenantArgument;
use cloud_terrastodon_gitea::GiteaTenantArgumentExt;
use cloud_terrastodon_gitea::fetch_all_gitea_organizations;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use std::io::Write;

#[derive(Args, Debug, Clone)]
pub struct GiteaOrgBrowseArgs {
    /// Tracked tenant URL or alias to query. Defaults to the active `tea` login.
    #[arg(long, default_value_t)]
    pub tenant: GiteaTenantArgument<'static>,
}

impl GiteaOrgBrowseArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenant = self.tenant.resolve().await?;
        let chosen = PickerTui::new()
            .pick_many_reloadable(async |invalidate| {
                fetch_all_gitea_organizations(&tenant)
                    .with_invalidation(invalidate)
                    .await
            })
            .await?;
        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        cloud_terrastodon_command::to_writer_pretty(&mut handle, &chosen)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
