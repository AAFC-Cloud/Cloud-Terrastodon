use crate::interactive::browse_service_principals;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;

/// Interactively browse Entra (Azure AD) service principals.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureEntraSpBrowseArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[facet(figue::named, default)]
    pub tenant: AzureTenantArgument<'static>,
}

impl AzureEntraSpBrowseArgs {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<()> {
        browse_service_principals(self.tenant.resolve().await?, auth_context).await
    }
}
