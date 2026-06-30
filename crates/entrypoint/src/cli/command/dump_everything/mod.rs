use crate::noninteractive::dump_everything;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use eyre::Result;

/// Dump all collected metadata to disk.
#[derive(facet::Facet, Debug, Clone, Default)]
pub struct DumpEverythingArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[facet(figue::named, default)]
    pub tenant: AzureTenantArgument<'static>,
}

impl DumpEverythingArgs {
    pub async fn invoke(self) -> Result<()> {
        dump_everything(self.tenant.resolve().await?).await?;
        Ok(())
    }
}
