use crate::agent::pool::entitlement::list::AzureDevOpsAgentPoolEntitlementListArgs;
use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// Azure DevOps agent pool entitlement-related commands (grouping for entitlement subcommands).
#[derive(Facet, Debug, Clone)]
pub struct AzureDevOpsAgentPoolEntitlementArgs {
    #[facet(figue::subcommand)]
    pub command: AzureDevOpsAgentPoolEntitlementCommand,
}

#[derive(Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureDevOpsAgentPoolEntitlementCommand {
    /// List agent pool entitlements (queues) in a project.
    List(AzureDevOpsAgentPoolEntitlementListArgs),
}

impl AzureDevOpsAgentPoolEntitlementArgs {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        match self.command {
            AzureDevOpsAgentPoolEntitlementCommand::List(args) => {
                args.invoke(auth_context, executor).await?
            }
        }

        Ok(())
    }
}
