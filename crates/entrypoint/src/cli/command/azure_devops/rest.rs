use clap::Args;
use eyre::Result;

/// Arguments for issuing raw Azure DevOps REST calls.
#[derive(Args, Debug, Clone)]
pub struct AzureDevOpsRestArgs {}

impl AzureDevOpsRestArgs {
    pub async fn invoke(self) -> Result<()> {
        // TODO: implement REST helpers.
        Ok(())
    }
}
