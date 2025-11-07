use super::terraform_source_add_imports::TerraformSourceAddImportsArgs;
use super::terraform_source_generate::TerraformSourceGenerateArgs;
use clap::Args;
use clap::Subcommand;
use eyre::Result;

/// Manage Terraform source files.
#[derive(Args, Debug, Clone)]
pub struct TerraformSourceArgs {
    #[command(subcommand)]
    pub command: TerraformSourceCommand,
}

impl TerraformSourceArgs {
    pub async fn invoke(self) -> Result<()> {
        self.command.invoke().await
    }
}

/// Operations available under `ct tf source`.
#[derive(Subcommand, Debug, Clone)]
pub enum TerraformSourceCommand {
    /// Create Terraform import definitions for selected resources.
    Generate(TerraformSourceGenerateArgs),
    /// Add Terraform import definitions to existing source files.
    #[command(name = "add-imports")]
    AddImports(TerraformSourceAddImportsArgs),
}

impl TerraformSourceCommand {
    pub async fn invoke(self) -> Result<()> {
        match self {
            TerraformSourceCommand::Generate(args) => args.invoke().await,
            TerraformSourceCommand::AddImports(args) => args.invoke().await,
        }
    }
}
