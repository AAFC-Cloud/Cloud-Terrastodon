use crate::noninteractive::audit_azure;
use clap::Args;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use eyre::Result;

/// Arguments for auditing Azure resources.
#[derive(Args, Debug, Clone)]
pub struct AzureAuditArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[arg(long, default_value_t)]
    pub tenant: AzureTenantArgument<'static>,
}

impl AzureAuditArgs {
    pub async fn invoke(self) -> Result<()> {
        audit_azure(self.tenant.resolve().await?).await
    }
}
