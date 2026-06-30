use crate::noninteractive::write_imports_for_all_resource_groups;
use crate::noninteractive::write_imports_for_all_role_assignments;
use crate::noninteractive::write_imports_for_all_security_groups;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use eyre::Result;

/// Write Terraform import definitions for all supported resources.
#[derive(facet::Facet, Debug, Clone, Default)]
pub struct WriteAllImportsArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[facet(figue::named, default)]
    pub tenant: AzureTenantArgument<'static>,
}

impl WriteAllImportsArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenant_id = self.tenant.resolve().await?;
        write_imports_for_all_resource_groups(tenant_id).await?;
        write_imports_for_all_security_groups(tenant_id).await?;
        write_imports_for_all_role_assignments(tenant_id).await?;
        Ok(())
    }
}
