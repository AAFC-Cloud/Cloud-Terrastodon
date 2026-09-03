use super::terraform_source_add_imports_cli::TerraformSourceAddImportsArgs;
use super::terraform_source_generate_cli::TerraformSourceGenerateArgs;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;

/// Manage Terraform source files.
#[derive(facet::Facet, Debug, Clone)]
pub struct TerraformSourceArgs {
    #[facet(figue::subcommand)]
    pub command: TerraformSourceCommand,
}

impl TerraformSourceArgs {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<()> {
        self.command.invoke(auth_context).await
    }
}

/// Operations available under `ct tf source`.
#[derive(facet::Facet, Debug, Clone)]
#[repr(u8)]
pub enum TerraformSourceCommand {
    /// Create Terraform import definitions for selected resources.
    Generate(TerraformSourceGenerateArgs),
    /// Add Terraform import definitions to existing source files.
    #[facet(rename = "add-imports")]
    AddImports(TerraformSourceAddImportsArgs),
}

impl TerraformSourceCommand {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<()> {
        match self {
            TerraformSourceCommand::Generate(args) => args.invoke(auth_context).await,
            TerraformSourceCommand::AddImports(args) => args.invoke(auth_context).await,
        }
    }
}
