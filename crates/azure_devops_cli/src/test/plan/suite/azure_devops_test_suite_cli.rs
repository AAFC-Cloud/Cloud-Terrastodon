use crate::test::plan::suite::cases::AzureDevOpsTestSuiteCaseArgs;
use crate::test::plan::suite::list::AzureDevOpsTestSuiteListArgs;
use crate::test::plan::suite::show::AzureDevOpsTestSuiteShowArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// Azure DevOps test suite-related commands.
#[derive(Facet, Debug, Clone)]
pub struct AzureDevOpsTestSuiteArgs {
    #[facet(figue::subcommand)]
    pub command: AzureDevOpsTestSuiteCommand,
}

#[derive(Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureDevOpsTestSuiteCommand {
    /// List Azure DevOps test suites in a plan.
    List(AzureDevOpsTestSuiteListArgs),

    /// Show details for a single Azure DevOps test suite.
    Show(AzureDevOpsTestSuiteShowArgs),

    /// Operations for test suite cases.
    Case(AzureDevOpsTestSuiteCaseArgs),
}

impl AzureDevOpsTestSuiteArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        match self.command {
            AzureDevOpsTestSuiteCommand::List(args) => Box::pin(args.invoke(executor)).await,
            AzureDevOpsTestSuiteCommand::Show(args) => Box::pin(args.invoke(executor)).await,
            AzureDevOpsTestSuiteCommand::Case(args) => Box::pin(args.invoke(executor)).await,
        }
    }
}
