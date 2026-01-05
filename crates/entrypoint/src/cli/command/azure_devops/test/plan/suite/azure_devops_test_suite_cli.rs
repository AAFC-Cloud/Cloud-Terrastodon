use crate::cli::azure_devops::test::plan::suite::cases::AzureDevOpsTestSuiteCaseArgs;
use crate::cli::azure_devops::test::plan::suite::list::AzureDevOpsTestSuiteListArgs;
use crate::cli::azure_devops::test::plan::suite::show::AzureDevOpsTestSuiteShowArgs;
use clap::Args;
use clap::Subcommand;
use eyre::Result;

/// Azure DevOps test suite-related commands.
#[derive(Args, Debug, Clone)]
pub struct AzureDevOpsTestSuiteArgs {
    #[command(subcommand)]
    pub command: AzureDevOpsTestSuiteCommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum AzureDevOpsTestSuiteCommand {
    /// List Azure DevOps test suites in a plan.
    List(AzureDevOpsTestSuiteListArgs),

    /// Show details for a single Azure DevOps test suite.
    Show(AzureDevOpsTestSuiteShowArgs),

    /// Operations for test suite cases.
    Case(AzureDevOpsTestSuiteCaseArgs),
}

impl AzureDevOpsTestSuiteArgs {
    pub async fn invoke(self) -> Result<()> {
        match self.command {
            AzureDevOpsTestSuiteCommand::List(args) => args.invoke().await?,
            AzureDevOpsTestSuiteCommand::Show(args) => args.invoke().await?,
            AzureDevOpsTestSuiteCommand::Case(args) => args.invoke().await?,
        }

        Ok(())
    }
}
