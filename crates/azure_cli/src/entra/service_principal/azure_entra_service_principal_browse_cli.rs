use crate::browse_service_principals;
use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;

/// Interactively browse Entra (Azure AD) service principals.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureEntraSpBrowseArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[facet(figue::named, default)]
    pub tenant: AzureTenantArgument<'static>,
}

impl AzureEntraSpBrowseArgs {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        let cancellation = executor.cancellation();
        let auth_context = self
            .tenant
            .bind_auth_context(auth_context, cancellation)
            .await?;
        let executor = &auth_context.auth_context.bind_executor(executor)?;
        browse_service_principals(&auth_context, executor).await
    }
}
