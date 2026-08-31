use crate::interactive::browse_policy_assignments;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;

/// Arguments for browsing Azure policy assignments interactively.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzurePolicyAssignmentBrowseArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[facet(figue::named, default)]
    pub tenant: AzureTenantArgument<'static>,
}

impl AzurePolicyAssignmentBrowseArgs {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<()> {
        browse_policy_assignments(self.tenant.resolve().await?, auth_context).await
    }
}
