use crate::noninteractive::prelude::write_imports_for_all_resource_groups;
use crate::noninteractive::prelude::write_imports_for_all_role_assignments;
use crate::noninteractive::prelude::write_imports_for_all_security_groups;
use clap::Args;
use cloud_terrastodon_azure::prelude::AzureTenantArgument;
use cloud_terrastodon_azure::prelude::AzureTenantArgumentExt;
use eyre::Result;

/// Write Terraform import definitions for all supported resources.
#[derive(Args, Debug, Clone, Default)]
pub struct WriteAllImportsArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[arg(long, default_value_t)]
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
