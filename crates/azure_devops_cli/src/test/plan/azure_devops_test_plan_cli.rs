use crate::test::plan::list::AzureDevOpsTestPlanListArgs;
use crate::test::plan::show::AzureDevOpsTestPlanShowArgs;
use crate::test::plan::suite::AzureDevOpsTestSuiteArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// Azure DevOps test plan-related commands.
#[derive(Facet, Debug, Clone)]
pub struct AzureDevOpsTestPlanArgs {
    #[facet(figue::subcommand)]
    pub command: AzureDevOpsTestPlanCommand,
}

#[derive(Facet, Debug, Clone)]
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
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        match self.command {
            AzureDevOpsTestPlanCommand::List(args) => Box::pin(args.invoke(executor)).await,
            AzureDevOpsTestPlanCommand::Show(args) => Box::pin(args.invoke(executor)).await,
            AzureDevOpsTestPlanCommand::Suite(args) => Box::pin(args.invoke(executor)).await,
        }
    }
}
