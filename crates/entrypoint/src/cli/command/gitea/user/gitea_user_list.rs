use clap::Args;
use cloud_terrastodon_gitea::GiteaTenantArgument;
use cloud_terrastodon_gitea::GiteaTenantArgumentExt;
use cloud_terrastodon_gitea::fetch_all_gitea_users;
use eyre::Result;
use std::io::Write;
use tracing::info;

#[derive(Args, Debug, Clone)]
pub struct GiteaUserListArgs {
    /// Tracked tenant URL or alias to query. Defaults to the active `tea` login.
    #[arg(long, default_value_t)]
    pub tenant: GiteaTenantArgument<'static>,
}

impl GiteaUserListArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenant = self.tenant.resolve().await?;
        info!(%tenant, "Fetching Gitea users");
        let users = fetch_all_gitea_users(&tenant).await?;
        info!(count = users.len(), "Fetched Gitea users");
        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        cloud_terrastodon_command::to_writer_pretty(&mut handle, &users)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
