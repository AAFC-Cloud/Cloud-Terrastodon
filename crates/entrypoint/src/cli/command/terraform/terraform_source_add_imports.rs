use clap::Args;
use eyre::Result;
use tracing::warn;

/// Add Terraform import blocks to existing source files.
#[derive(Args, Debug, Clone)]
pub struct TerraformSourceAddImportsArgs {}

impl TerraformSourceAddImportsArgs {
    pub async fn invoke(self) -> Result<()> {
        warn!("ct tf source add-imports is not implemented yet");
        Ok(())
    }
}
