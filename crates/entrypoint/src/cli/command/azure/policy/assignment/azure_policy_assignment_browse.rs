use crate::interactive::browse_policy_assignments;
use clap::Args;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use eyre::Result;

/// Arguments for browsing Azure policy assignments interactively.
#[derive(Args, Debug, Clone)]
pub struct AzurePolicyAssignmentBrowseArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[arg(long, default_value_t)]
    pub tenant: AzureTenantArgument<'static>,
}

impl AzurePolicyAssignmentBrowseArgs {
    pub async fn invoke(self) -> Result<()> {
        browse_policy_assignments(self.tenant.resolve().await?).await
    }
}
