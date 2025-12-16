pub mod terraform_audit;
pub mod terraform_command;
pub mod terraform_reflow;
pub mod terraform_source;
pub mod terraform_source_add_imports;
pub mod terraform_source_generate;

use clap::Args;
use eyre::Result;
pub use terraform_command::TerraformCommand;

/// Arguments for Terraform-specific operations.
#[derive(Args, Debug, Clone)]
pub struct TerraformArgs {
    #[command(subcommand)]
    pub command: TerraformCommand,
}

impl TerraformArgs {
    pub async fn invoke(self) -> Result<()> {
        self.command.invoke().await
    }
}
