use crate::noninteractive::audit_azure;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;

/// Arguments for auditing Azure resources.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureAuditArgs {
    /// Tracked tenant id or alias to query. Defaults to the selected
    /// workload-identity/browser tenant, then the Azure CLI tenant.
    #[facet(figue::named, default)]
    pub tenant: AzureTenantArgument<'static>,
}

impl AzureAuditArgs {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<()> {
        let auth_context = self.tenant.bind_auth_context(auth_context).await?;
        audit_azure(&auth_context).await
    }
}
