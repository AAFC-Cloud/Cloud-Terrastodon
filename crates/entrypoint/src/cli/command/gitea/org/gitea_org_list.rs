use clap::Args;
use cloud_terrastodon_gitea::GiteaTenantArgument;
use cloud_terrastodon_gitea::GiteaTenantArgumentExt;
use cloud_terrastodon_gitea::fetch_all_gitea_organizations;
use eyre::Result;
use std::io::Write;
use tracing::info;

#[derive(Args, Debug, Clone)]
pub struct GiteaOrgListArgs {
    /// Tracked tenant URL or alias to query. Defaults to the active `tea` login.
    #[arg(long, default_value_t)]
    pub tenant: GiteaTenantArgument<'static>,
}

impl GiteaOrgListArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenant = self.tenant.resolve().await?;
        info!(%tenant, "Fetching Gitea organizations");
        let organizations = fetch_all_gitea_organizations(&tenant).await?;
        info!(count = organizations.len(), "Fetched Gitea organizations");
        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        serde_json::to_writer_pretty(&mut handle, &organizations)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
