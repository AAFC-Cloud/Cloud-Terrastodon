use super::terraform_audit_cli::TerraformAuditArgs;
use super::terraform_reflow_cli::TerraformReflowArgs;
use super::terraform_show_cli::TerraformShowArgs;
use super::terraform_source_cli::TerraformSourceArgs;
use crate::cli::terraform::terraform_apply_cli::TerraformApplyArgs;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// Terraform-specific commands.
#[derive(Facet, Debug, Clone)]
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
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<()> {
        let cancellation = executor.cancellation();
        match self {
            TerraformCommand::Audit(args) => args.invoke(cancellation).await,
            TerraformCommand::Source(args) => args.invoke(executor).await,
            TerraformCommand::Reflow(args) => args.invoke(executor).await,
            TerraformCommand::Show(args) => args.invoke(executor).await,
            TerraformCommand::Apply(args) => args.invoke(executor).await,
        }
    }
}
