use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_gitea::GiteaTenantArgument;
use cloud_terrastodon_gitea::GiteaTenantArgumentExt;
use cloud_terrastodon_gitea::fetch_all_gitea_organizations;
use eyre::Result;
use std::io::Write;
use tracing::info;

#[derive(facet::Facet, Debug, Clone)]
pub struct GiteaOrgListArgs {
    /// Tracked tenant URL or alias to query. Defaults to the active `tea` login.
    #[facet(figue::named, default, proxy = String)]
    pub tenant: GiteaTenantArgument<'static>,
}

impl GiteaOrgListArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<()> {
        let tenant = self.tenant.resolve(executor).await?;
        info!(%tenant, "Fetching Gitea organizations");
        let organizations = fetch_all_gitea_organizations(&tenant, executor).await?;
        info!(count = organizations.len(), "Fetched Gitea organizations");
        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        facet_json::to_writer_std_pretty(&mut handle, &organizations)
            .map_err(|error| eyre::eyre!("{error:?}"))?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
