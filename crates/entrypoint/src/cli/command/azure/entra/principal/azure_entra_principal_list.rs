use clap::Args;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::fetch_all_principals;
use eyre::Result;
use std::io::Write;
use tracing::info;

/// List Entra (Azure AD) principals.
#[derive(Args, Debug, Clone)]
pub struct AzureEntraPrincipalListArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[arg(long, default_value_t)]
    pub tenant: AzureTenantArgument<'static>,
}

impl AzureEntraPrincipalListArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenant_id = self.tenant.resolve().await?;
        info!(%tenant_id, "Fetching Entra principals");
        let principals = fetch_all_principals(tenant_id).await?;
        let mut principals = principals.values().collect::<Vec<_>>();
        principals.sort_unstable_by_key(|principal| principal.id().to_string());

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        serde_json::to_writer_pretty(&mut handle, &principals)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
