use crate::noninteractive::write_imports_for_all_resource_groups;
use crate::noninteractive::write_imports_for_all_role_assignments;
use crate::noninteractive::write_imports_for_all_security_groups;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;

/// Write Terraform import definitions for all supported resources.
#[derive(facet::Facet, Debug, Clone, Default)]
pub struct WriteAllImportsArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[facet(figue::named, default)]
    pub tenant: AzureTenantArgument<'static>,
}

impl WriteAllImportsArgs {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<()> {
        let auth_context = self.tenant.bind_auth_context(auth_context).await?;
        write_imports_for_all_resource_groups(&auth_context).await?;
        write_imports_for_all_security_groups(&auth_context).await?;
        write_imports_for_all_role_assignments(&auth_context).await?;
        Ok(())
    }
}
