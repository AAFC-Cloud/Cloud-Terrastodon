use crate::cli::azure_devops::test::plan::suite::cases::list::AzureDevOpsTestSuiteCaseListArgs;
use crate::cli::azure_devops::test::plan::suite::cases::show::AzureDevOpsTestSuiteCaseShowArgs;
use clap::Args;
use clap::Subcommand;
use eyre::Result;

/// Azure DevOps test suite cases-related commands.
#[derive(Args, Debug, Clone)]
pub struct AzureDevOpsTestSuiteCaseArgs {
    #[command(subcommand)]
    pub command: AzureDevOpsTestSuiteCaseCommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum AzureDevOpsTestSuiteCaseCommand {
    /// List Azure DevOps test cases in a suite.
    List(AzureDevOpsTestSuiteCaseListArgs),

    /// Show details for a single Azure DevOps test case.
    Show(AzureDevOpsTestSuiteCaseShowArgs),
}

impl AzureDevOpsTestSuiteCaseArgs {
    pub async fn invoke(self) -> Result<()> {
        match self.command {
            AzureDevOpsTestSuiteCaseCommand::List(args) => args.invoke().await?,
            AzureDevOpsTestSuiteCaseCommand::Show(args) => args.invoke().await?,
        }

        Ok(())
    }
}
