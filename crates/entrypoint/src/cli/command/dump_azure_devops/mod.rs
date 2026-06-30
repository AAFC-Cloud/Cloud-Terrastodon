use crate::noninteractive::dump_azure_devops;
use eyre::Result;

/// Dump Azure DevOps metadata to disk.
#[derive(facet::Facet, Debug, Clone, Default)]
pub struct DumpAzureDevOpsArgs;

impl DumpAzureDevOpsArgs {
    pub async fn invoke(self) -> Result<()> {
        dump_azure_devops().await?;
        Ok(())
    }
}
