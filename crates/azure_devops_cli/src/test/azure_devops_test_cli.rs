use crate::test::plan::AzureDevOpsTestPlanArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// Azure DevOps test-related commands.
#[derive(Facet, Debug, Clone)]
pub struct AzureDevOpsTestArgs {
    #[facet(figue::subcommand)]
    pub command: AzureDevOpsTestCommand,
}

#[derive(Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureDevOpsTestCommand {
    /// Test plan operations.
    Plan(AzureDevOpsTestPlanArgs),
}

impl AzureDevOpsTestArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        match self.command {
            AzureDevOpsTestCommand::Plan(args) => Box::pin(args.invoke(executor)).await,
        }
    }
}
