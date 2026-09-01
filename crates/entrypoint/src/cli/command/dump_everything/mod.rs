use crate::noninteractive::dump_everything;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;

/// Dump all collected metadata to disk.
#[derive(facet::Facet, Debug, Clone, Default)]
pub struct DumpEverythingArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[facet(figue::named, default)]
    pub tenant: AzureTenantArgument<'static>,
}

impl DumpEverythingArgs {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<()> {
        let auth_context = self.tenant.bind_auth_context(auth_context).await?;
        dump_everything(&auth_context).await?;
        Ok(())
    }
}
