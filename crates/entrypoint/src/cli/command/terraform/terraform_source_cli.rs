use super::terraform_source_add_imports_cli::TerraformSourceAddImportsArgs;
use super::terraform_source_generate_cli::TerraformSourceGenerateArgs;
use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// Manage Terraform source files.
#[derive(Facet, Debug, Clone)]
pub struct TerraformSourceArgs {
    #[facet(figue::subcommand)]
    pub command: TerraformSourceCommand,
}

impl TerraformSourceArgs {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        self.command.invoke(auth_context, executor).await
    }
}

/// Operations available under `ct tf source`.
#[derive(Facet, Debug, Clone)]
#[repr(u8)]
pub enum TerraformSourceCommand {
    /// Create Terraform import definitions for selected resources.
    Generate(TerraformSourceGenerateArgs),
    /// Add Terraform import definitions to existing source files.
    #[facet(rename = "add-imports")]
    AddImports(TerraformSourceAddImportsArgs),
}

impl TerraformSourceCommand {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        match self {
            TerraformSourceCommand::Generate(args) => args.invoke(auth_context, executor).await,
            TerraformSourceCommand::AddImports(args) => args.invoke(auth_context, executor).await,
        }
    }
}
