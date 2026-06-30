use crate::cli::azure_devops::test::plan::list::AzureDevOpsTestPlanListArgs;
use crate::cli::azure_devops::test::plan::show::AzureDevOpsTestPlanShowArgs;
use crate::cli::azure_devops::test::plan::suite::AzureDevOpsTestSuiteArgs;
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
    pub async fn invoke(self) -> Result<()> {
        match self.command {
            AzureDevOpsTestPlanCommand::List(args) => args.invoke().await?,
            AzureDevOpsTestPlanCommand::Show(args) => args.invoke().await?,
            AzureDevOpsTestPlanCommand::Suite(args) => args.invoke().await?,
        }

        Ok(())
    }
}
