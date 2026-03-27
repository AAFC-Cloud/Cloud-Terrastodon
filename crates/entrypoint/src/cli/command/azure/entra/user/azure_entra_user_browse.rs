use crate::interactive::prelude::browse_users;
use clap::Args;
use cloud_terrastodon_azure::prelude::AzureTenantArgument;
use cloud_terrastodon_azure::prelude::AzureTenantArgumentExt;
use eyre::Result;

/// Interactively browse Entra (Azure AD) users.
#[derive(Args, Debug, Clone)]
pub struct AzureEntraUserBrowseArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[arg(long, default_value_t)]
    pub tenant: AzureTenantArgument<'static>,
}

impl AzureEntraUserBrowseArgs {
    pub async fn invoke(self) -> Result<()> {
        browse_users(self.tenant.resolve().await?).await
    }
}
