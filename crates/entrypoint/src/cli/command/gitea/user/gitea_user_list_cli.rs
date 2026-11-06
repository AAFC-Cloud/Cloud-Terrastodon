use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_gitea::GiteaTenantArgument;
use cloud_terrastodon_gitea::GiteaTenantArgumentExt;
use cloud_terrastodon_gitea::fetch_all_gitea_users;
use eyre::Result;
use facet::Facet;
use std::io::Write;
use tracing::info;

#[derive(Facet, Debug, Clone)]
pub struct GiteaUserListArgs {
    /// Tracked tenant URL or alias to query. Defaults to the active `tea` login.
    #[facet(figue::named, default, proxy = String)]
    pub tenant: GiteaTenantArgument<'static>,
}

impl GiteaUserListArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<()> {
        let tenant = self.tenant.resolve(executor).await?;
        info!(%tenant, "Fetching Gitea users");
        let users = fetch_all_gitea_users(&tenant, executor).await?;
        info!(count = users.len(), "Fetched Gitea users");
        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        facet_json::to_writer_std_pretty(&mut handle, &users)
            .map_err(|error| eyre::eyre!("{error:?}"))?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
