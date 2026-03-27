use crate::noninteractive::prelude::perform_import;
use crate::noninteractive::prelude::process_generated;
use clap::Args;
use cloud_terrastodon_azure::prelude::AzureTenantArgument;
use cloud_terrastodon_azure::prelude::AzureTenantArgumentExt;
use eyre::Result;

/// Perform code-generation from existing import definitions.
#[derive(Args, Debug, Clone, Default)]
pub struct PerformCodeGenerationFromImportsArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[arg(long, default_value_t)]
    pub tenant: AzureTenantArgument<'static>,
}

impl PerformCodeGenerationFromImportsArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenant_id = self.tenant.resolve().await?;
        perform_import().await?;
        process_generated(tenant_id).await?;
        Ok(())
    }
}
