use super::terraform_audit::TerraformAuditArgs;
use super::terraform_reflow::TerraformReflowArgs;
use super::terraform_source::TerraformSourceArgs;
use clap::Subcommand;
use eyre::Result;

/// Terraform-specific commands.
#[derive(Subcommand, Debug, Clone)]
pub enum TerraformCommand {
    /// Identify and report Terraform provider issues.
    ///
    /// Identify if any providers have been specified as required but are not being used.
    ///
    /// Identify if any providers are not using the latest version.
    Audit(TerraformAuditArgs),
    /// Manage operations on generated Terraform source files.
    #[command(alias = "src")]
    Source(TerraformSourceArgs),
    /// Reflow generated Terraform source files.
    Reflow(TerraformReflowArgs),
}

impl TerraformCommand {
    pub async fn invoke(self) -> Result<()> {
        match self {
            TerraformCommand::Audit(args) => args.invoke().await,
            TerraformCommand::Source(args) => args.invoke().await,
            TerraformCommand::Reflow(args) => args.invoke().await,
        }
    }
}
