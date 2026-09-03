pub mod terraform_apply_cli;
pub mod terraform_audit_cli;
pub mod terraform_command_cli;
pub mod terraform_reflow_cli;
pub mod terraform_show_cli;
pub mod terraform_source_add_imports_cli;
pub mod terraform_source_cli;
pub mod terraform_source_generate_cli;

use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;
pub use terraform_command_cli::TerraformCommand;

/// Arguments for Terraform-specific operations.
#[derive(facet::Facet, Debug, Clone)]
pub struct TerraformArgs {
    #[facet(figue::subcommand)]
    pub command: TerraformCommand,
}

impl TerraformArgs {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<()> {
        self.command.invoke(auth_context).await
    }
}
