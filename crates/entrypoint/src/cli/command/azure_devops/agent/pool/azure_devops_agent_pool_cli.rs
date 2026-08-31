use crate::cli::azure_devops::agent::pool::entitlement::AzureDevOpsAgentPoolEntitlementArgs;
use crate::cli::azure_devops::agent::pool::list::AzureDevOpsAgentPoolListArgs;
use crate::cli::azure_devops::agent::pool::summary::AzureDevOpsAgentPoolSummaryArgs;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;

/// Azure DevOps agent pool-related commands (grouping for pool subcommands).
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureDevOpsAgentPoolArgs {
    #[facet(figue::subcommand)]
    pub command: AzureDevOpsAgentPoolCommand,
}

#[derive(facet::Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureDevOpsAgentPoolCommand {
    /// List Azure DevOps agent pools in the organization.
    List(AzureDevOpsAgentPoolListArgs),
    /// Agent pool entitlement-related commands.
    Entitlement(AzureDevOpsAgentPoolEntitlementArgs),
    /// Summary of agent pools and projects that use them.
    Summary(AzureDevOpsAgentPoolSummaryArgs),
}

impl AzureDevOpsAgentPoolArgs {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<()> {
        match self.command {
            AzureDevOpsAgentPoolCommand::List(args) => args.invoke().await?,
            AzureDevOpsAgentPoolCommand::Entitlement(args) => args.invoke(auth_context).await?,
            AzureDevOpsAgentPoolCommand::Summary(args) => args.invoke(auth_context).await?,
        }

        Ok(())
    }
}
