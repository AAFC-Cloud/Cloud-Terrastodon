use crate::test::plan::AzureDevOpsTestPlanArgs;
use cloud_terrastodon_auth::AuthContext;
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
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        match self.command {
            AzureDevOpsTestCommand::Plan(args) => args.invoke(auth_context, executor).await?,
        }

        Ok(())
    }
}
