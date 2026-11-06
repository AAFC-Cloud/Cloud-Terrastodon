use crate::test::plan::suite::cases::AzureDevOpsTestSuiteCaseArgs;
use crate::test::plan::suite::list::AzureDevOpsTestSuiteListArgs;
use crate::test::plan::suite::show::AzureDevOpsTestSuiteShowArgs;
use cloud_terrastodon_auth::AuthContext;
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
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        match self.command {
            AzureDevOpsTestSuiteCommand::List(args) => args.invoke(auth_context, executor).await?,
            AzureDevOpsTestSuiteCommand::Show(args) => args.invoke(auth_context, executor).await?,
            AzureDevOpsTestSuiteCommand::Case(args) => args.invoke(auth_context, executor).await?,
        }

        Ok(())
    }
}
