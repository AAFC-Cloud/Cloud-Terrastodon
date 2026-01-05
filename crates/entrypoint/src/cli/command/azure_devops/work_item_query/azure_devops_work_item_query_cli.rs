use crate::cli::azure_devops::work_item_query::invoke::AzureDevOpsWorkItemQueryInvokeArgs;
use crate::cli::azure_devops::work_item_query::list::AzureDevOpsWorkItemQueryListArgs;
use clap::Args;
use clap::Subcommand;
use eyre::Result;

/// Azure DevOps work item query-related commands.
#[derive(Args, Debug, Clone)]
pub struct AzureDevOpsWorkItemQueryArgs {
    #[command(subcommand)]
    pub command: AzureDevOpsWorkItemQueryCommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum AzureDevOpsWorkItemQueryCommand {
    /// List queries in a project.
    List(AzureDevOpsWorkItemQueryListArgs),
    /// Invoke a query and print results.
    Invoke(AzureDevOpsWorkItemQueryInvokeArgs),
}

impl AzureDevOpsWorkItemQueryArgs {
    pub async fn invoke(self) -> Result<()> {
        match self.command {
            AzureDevOpsWorkItemQueryCommand::List(args) => args.invoke().await?,
            AzureDevOpsWorkItemQueryCommand::Invoke(args) => args.invoke().await?,
        }

        Ok(())
    }
}
