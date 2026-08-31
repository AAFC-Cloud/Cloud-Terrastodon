use crate::cli::azure_devops::work_item_query::invoke::AzureDevOpsWorkItemQueryInvokeArgs;
use crate::cli::azure_devops::work_item_query::list::AzureDevOpsWorkItemQueryListArgs;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;

/// Azure DevOps work item query-related commands.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureDevOpsWorkItemQueryArgs {
    #[facet(figue::subcommand)]
    pub command: AzureDevOpsWorkItemQueryCommand,
}

#[derive(facet::Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureDevOpsWorkItemQueryCommand {
    /// List queries in a project.
    List(AzureDevOpsWorkItemQueryListArgs),
    /// Invoke a query and print results.
    Invoke(AzureDevOpsWorkItemQueryInvokeArgs),
}

impl AzureDevOpsWorkItemQueryArgs {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<()> {
        match self.command {
            AzureDevOpsWorkItemQueryCommand::List(args) => args.invoke(auth_context).await?,
            AzureDevOpsWorkItemQueryCommand::Invoke(args) => args.invoke().await?,
        }

        Ok(())
    }
}
