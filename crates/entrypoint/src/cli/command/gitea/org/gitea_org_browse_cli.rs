use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_gitea::GiteaTenantArgument;
use cloud_terrastodon_gitea::GiteaTenantArgumentExt;
use cloud_terrastodon_gitea::fetch_all_gitea_organizations;
use cloud_terrastodon_user_input::PickResultExt;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use facet::Facet;
use std::io::Write;

#[derive(Facet, Debug, Clone)]
pub struct GiteaOrgBrowseArgs {
    /// Tracked tenant URL or alias to query. Defaults to the active `tea` login.
    #[facet(figue::named, default, proxy = String)]
    pub tenant: GiteaTenantArgument<'static>,
}

impl GiteaOrgBrowseArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<()> {
        let tenant = self.tenant.resolve(executor).await?;
        let (chosen, maybe_error) = PickerTui::<_>::new()
            .pick_many_reloadable(executor.cancellation(), async |child, invalidate| {
                let executor = &executor.with_cancellation_scope(child);
                fetch_all_gitea_organizations(&tenant, executor)
                    .with_invalidation(invalidate)
                    .await
            })
            .await
            .into_chosen_and_maybe_error()?;
        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        facet_json::to_writer_std_pretty(&mut handle, &chosen)
            .map_err(|error| eyre::eyre!("{error:?}"))?;
        handle.write_all(b"\n")?;
        maybe_error
    }
}
