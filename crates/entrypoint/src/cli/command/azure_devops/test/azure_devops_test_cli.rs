use crate::cli::azure_devops::test::plan::AzureDevOpsTestPlanArgs;
use clap::Args;
use clap::Subcommand;
use eyre::Result;

/// Azure DevOps test-related commands.
#[derive(Args, Debug, Clone)]
pub struct AzureDevOpsTestArgs {
    #[command(subcommand)]
    pub command: AzureDevOpsTestCommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum AzureDevOpsTestCommand {
    /// Test plan operations.
    Plan(AzureDevOpsTestPlanArgs),
}

impl AzureDevOpsTestArgs {
    pub async fn invoke(self) -> Result<()> {
        match self.command {
            AzureDevOpsTestCommand::Plan(args) => args.invoke().await?,
        }

        Ok(())
    }
}
