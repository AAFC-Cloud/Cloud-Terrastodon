use clap::Args;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::fetch_all_service_principals;
use eyre::Result;
use std::io::Write;
use tracing::info;

/// List Entra (Azure AD) service principals.
#[derive(Args, Debug, Clone)]
pub struct AzureEntraSpListArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[arg(long, default_value_t)]
    pub tenant: AzureTenantArgument<'static>,
}

impl AzureEntraSpListArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenant_id = self.tenant.resolve().await?;
        info!("Fetching service principals");
        let sps = fetch_all_service_principals(tenant_id).await?;
        let stdout = std::io::stdout();
        let mut out = stdout.lock();
        for sp in sps {
            writeln!(
                out,
                "{} {:64} {} {}",
                sp.id,
                sp.display_name,
                sp.app_id,
                sp.service_principal_names.join(",")
            )?;
        }
        Ok(())
    }
}
