use super::terraform_audit::TerraformAuditArgs;
use super::terraform_reflow::TerraformReflowArgs;
use super::terraform_show::TerraformShowArgs;
use super::terraform_source::TerraformSourceArgs;
use crate::cli::terraform::terraform_apply::TerraformApplyArgs;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;

/// Terraform-specific commands.
#[derive(facet::Facet, Debug, Clone)]
#[repr(u8)]
pub enum TerraformCommand {
    /// Identify and report Terraform provider issues.
    ///
    /// Identify if any providers have been specified as required but are not being used.
    ///
    /// Identify if any providers are not using the latest version.
    Audit(TerraformAuditArgs),
    /// Manage operations on generated Terraform source files.
    #[facet(figue::alias = "src")]
    Source(TerraformSourceArgs),
    /// Reflow generated Terraform source files.
    Reflow(TerraformReflowArgs),
    /// Show a Terraform plan (supports .tfplan or .json)
    Show(TerraformShowArgs),
    /// Apply Terraform source files.
    Apply(TerraformApplyArgs),
}

impl TerraformCommand {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<()> {
        match self {
            TerraformCommand::Audit(args) => args.invoke().await,
            TerraformCommand::Source(args) => args.invoke(auth_context).await,
            TerraformCommand::Reflow(args) => args.invoke(auth_context).await,
            TerraformCommand::Show(args) => args.invoke(auth_context).await,
            TerraformCommand::Apply(args) => args.invoke(auth_context).await,
        }
    }
}
