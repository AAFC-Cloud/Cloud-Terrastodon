use crate::cli::azure_devops::test::plan::list::AzureDevOpsTestPlanListArgs;
use crate::cli::azure_devops::test::plan::show::AzureDevOpsTestPlanShowArgs;
use crate::cli::azure_devops::test::plan::suite::AzureDevOpsTestSuiteArgs;
use clap::Args;
use clap::Subcommand;
use eyre::Result;

/// Azure DevOps test plan-related commands.
#[derive(Args, Debug, Clone)]
pub struct AzureDevOpsTestPlanArgs {
    #[command(subcommand)]
    pub command: AzureDevOpsTestPlanCommand,
}

#[derive(Subcommand, Debug, Clone)]
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
