use clap::Args;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::fetch_all_users;
use eyre::Result;
use std::io::Write;
use tracing::info;

/// List Entra (Azure AD) users.
#[derive(Args, Debug, Clone)]
pub struct AzureEntraUserListArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[arg(long, default_value_t)]
    pub tenant: AzureTenantArgument<'static>,
}

impl AzureEntraUserListArgs {
    pub async fn invoke(self) -> Result<()> {
        info!("Fetching users");
        let users = fetch_all_users(self.tenant.resolve().await?).await?;

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        serde_json::to_writer_pretty(&mut handle, &users)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
