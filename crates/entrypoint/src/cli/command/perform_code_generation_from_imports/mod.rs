use crate::noninteractive::perform_import;
use crate::noninteractive::process_generated;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;

/// Perform code-generation from existing import definitions.
#[derive(facet::Facet, Debug, Clone, Default)]
pub struct PerformCodeGenerationFromImportsArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[facet(figue::named, default)]
    pub tenant: AzureTenantArgument<'static>,
}

impl PerformCodeGenerationFromImportsArgs {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<()> {
        let auth_context = self.tenant.bind_auth_context(auth_context).await?;
        perform_import().await?;
        process_generated(&auth_context).await?;
        Ok(())
    }
}
