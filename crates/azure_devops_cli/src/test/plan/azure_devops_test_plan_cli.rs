use crate::test::plan::list::AzureDevOpsTestPlanListArgs;
use crate::test::plan::show::AzureDevOpsTestPlanShowArgs;
use crate::test::plan::suite::AzureDevOpsTestSuiteArgs;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;

/// Azure DevOps test plan-related commands.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureDevOpsTestPlanArgs {
    #[facet(figue::subcommand)]
    pub command: AzureDevOpsTestPlanCommand,
}

#[derive(facet::Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureDevOpsTestPlanCommand {
    /// List Azure DevOps test plans in a project.
    List(AzureDevOpsTestPlanListArgs),

    /// Show details for a single Azure DevOps test plan.
    Show(AzureDevOpsTestPlanShowArgs),

    /// Test suite operations.
    Suite(AzureDevOpsTestSuiteArgs),
}

impl AzureDevOpsTestPlanArgs {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        cancellation: &CancellationScope,
    ) -> Result<()> {
        match self.command {
            AzureDevOpsTestPlanCommand::List(args) => {
                args.invoke(auth_context, cancellation).await?
            }
            AzureDevOpsTestPlanCommand::Show(args) => {
                args.invoke(auth_context, cancellation).await?
            }
            AzureDevOpsTestPlanCommand::Suite(args) => {
                args.invoke(auth_context, cancellation).await?
            }
        }

        Ok(())
    }
}
