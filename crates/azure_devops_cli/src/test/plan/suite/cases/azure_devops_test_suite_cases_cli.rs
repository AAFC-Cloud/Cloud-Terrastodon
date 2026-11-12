use crate::test::plan::suite::cases::list::AzureDevOpsTestSuiteCaseListArgs;
use crate::test::plan::suite::cases::show::AzureDevOpsTestSuiteCaseShowArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// Azure DevOps test suite cases-related commands.
#[derive(Facet, Debug, Clone)]
pub struct AzureDevOpsTestSuiteCaseArgs {
    #[facet(figue::subcommand)]
    pub command: AzureDevOpsTestSuiteCaseCommand,
}

#[derive(Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureDevOpsTestSuiteCaseCommand {
    /// List Azure DevOps test cases in a suite.
    List(AzureDevOpsTestSuiteCaseListArgs),

    /// Show details for a single Azure DevOps test case.
    Show(AzureDevOpsTestSuiteCaseShowArgs),
}

impl AzureDevOpsTestSuiteCaseArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        match self.command {
            AzureDevOpsTestSuiteCaseCommand::List(args) => Box::pin(args.invoke(executor)).await,
            AzureDevOpsTestSuiteCaseCommand::Show(args) => Box::pin(args.invoke(executor)).await,
        }
    }
}
