use crate::interactive::browse_users;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use eyre::Result;

/// Interactively browse Entra (Azure AD) users.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureEntraUserBrowseArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[facet(figue::named, default)]
    pub tenant: AzureTenantArgument<'static>,
}

impl AzureEntraUserBrowseArgs {
    pub async fn invoke(self) -> Result<()> {
        browse_users(self.tenant.resolve().await?).await
    }
}
