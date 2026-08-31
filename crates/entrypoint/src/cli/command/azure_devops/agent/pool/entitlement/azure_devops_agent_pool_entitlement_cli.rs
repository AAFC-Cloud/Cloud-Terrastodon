use crate::cli::azure_devops::agent::pool::entitlement::list::AzureDevOpsAgentPoolEntitlementListArgs;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;

/// Azure DevOps agent pool entitlement-related commands (grouping for entitlement subcommands).
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureDevOpsAgentPoolEntitlementArgs {
    #[facet(figue::subcommand)]
    pub command: AzureDevOpsAgentPoolEntitlementCommand,
}

#[derive(facet::Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureDevOpsAgentPoolEntitlementCommand {
    /// List agent pool entitlements (queues) in a project.
    List(AzureDevOpsAgentPoolEntitlementListArgs),
}

impl AzureDevOpsAgentPoolEntitlementArgs {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<()> {
        match self.command {
            AzureDevOpsAgentPoolEntitlementCommand::List(args) => args.invoke(auth_context).await?,
        }

        Ok(())
    }
}
