pub mod terraform_apply;
pub mod terraform_audit;
pub mod terraform_command;
pub mod terraform_reflow;
pub mod terraform_show;
pub mod terraform_source;
pub mod terraform_source_add_imports;
pub mod terraform_source_generate;

use eyre::Result;
pub use terraform_command::TerraformCommand;

/// Arguments for Terraform-specific operations.
#[derive(facet::Facet, Debug, Clone)]
pub struct TerraformArgs {
    #[facet(figue::subcommand)]
    pub command: TerraformCommand,
}

impl TerraformArgs {
    pub async fn invoke(self) -> Result<()> {
        self.command.invoke().await
    }
}
