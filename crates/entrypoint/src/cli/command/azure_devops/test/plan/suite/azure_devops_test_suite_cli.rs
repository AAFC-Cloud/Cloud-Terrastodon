use crate::cli::azure_devops::test::plan::suite::cases::AzureDevOpsTestSuiteCaseArgs;
use crate::cli::azure_devops::test::plan::suite::list::AzureDevOpsTestSuiteListArgs;
use crate::cli::azure_devops::test::plan::suite::show::AzureDevOpsTestSuiteShowArgs;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;

/// Azure DevOps test suite-related commands.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureDevOpsTestSuiteArgs {
    #[facet(figue::subcommand)]
    pub command: AzureDevOpsTestSuiteCommand,
}

#[derive(facet::Facet, Debug, Clone)]
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
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<()> {
        match self.command {
            AzureDevOpsTestSuiteCommand::List(args) => args.invoke(auth_context).await?,
            AzureDevOpsTestSuiteCommand::Show(args) => args.invoke(auth_context).await?,
            AzureDevOpsTestSuiteCommand::Case(args) => args.invoke().await?,
        }

        Ok(())
    }
}
