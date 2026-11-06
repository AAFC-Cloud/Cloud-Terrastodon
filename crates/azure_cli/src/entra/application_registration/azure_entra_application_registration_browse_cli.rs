use crate::browse_application_registrations;
use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// Interactively browse Entra (Azure AD) application registrations.
#[derive(Facet, Debug, Clone)]
pub struct AzureEntraApplicationRegistrationBrowseArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[facet(figue::named, default)]
    pub tenant: AzureTenantArgument<'static>,
}

impl AzureEntraApplicationRegistrationBrowseArgs {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        let cancellation = executor.cancellation();
        browse_application_registrations(
            self.tenant.resolve(cancellation).await?,
            auth_context,
            executor,
        )
        .await
    }
}
