use crate::cli::azure_devops::test::plan::suite::cases::list::AzureDevOpsTestSuiteCaseListArgs;
use crate::cli::azure_devops::test::plan::suite::cases::show::AzureDevOpsTestSuiteCaseShowArgs;
use eyre::Result;

/// Azure DevOps test suite cases-related commands.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureDevOpsTestSuiteCaseArgs {
    #[facet(figue::subcommand)]
    pub command: AzureDevOpsTestSuiteCaseCommand,
}

#[derive(facet::Facet, Debug, Clone)]
#[repr(u8)]
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
