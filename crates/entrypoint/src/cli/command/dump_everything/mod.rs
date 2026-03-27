use crate::noninteractive::dump_everything;
use clap::Args;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use eyre::Result;

/// Dump all collected metadata to disk.
#[derive(Args, Debug, Clone, Default)]
pub struct DumpEverythingArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[arg(long, default_value_t)]
    pub tenant: AzureTenantArgument<'static>,
}

impl DumpEverythingArgs {
    pub async fn invoke(self) -> Result<()> {
        dump_everything(self.tenant.resolve().await?).await?;
        Ok(())
    }
}
