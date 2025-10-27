pub mod terraform_command;

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
