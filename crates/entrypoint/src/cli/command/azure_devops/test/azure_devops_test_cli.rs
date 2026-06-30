use crate::cli::azure_devops::test::plan::AzureDevOpsTestPlanArgs;
use eyre::Result;

/// Azure DevOps test-related commands.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureDevOpsTestArgs {
    #[facet(figue::subcommand)]
    pub command: AzureDevOpsTestCommand,
}

#[derive(facet::Facet, Debug, Clone)]
#[repr(u8)]
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
