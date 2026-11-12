use crate::agent::pool::entitlement::list::AzureDevOpsAgentPoolEntitlementListArgs;
use cloud_terrastodon_app::CliOutput;
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
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        match self.command {
            AzureDevOpsAgentPoolEntitlementCommand::List(args) => {
                Box::pin(args.invoke(executor)).await
            }
        }
    }
}
