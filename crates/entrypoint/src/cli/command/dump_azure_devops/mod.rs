use crate::noninteractive::prelude::dump_azure_devops;
use clap::Args;
use eyre::Result;

/// Dump Azure DevOps metadata to disk.
#[derive(Args, Debug, Clone, Default)]
pub struct DumpAzureDevOpsArgs;

impl DumpAzureDevOpsArgs {
    pub async fn invoke(self) -> Result<()> {
        dump_azure_devops().await?;
        Ok(())
    }
}
