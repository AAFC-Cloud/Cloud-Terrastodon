pub mod agent;
pub mod agent_package;
pub mod audit;
pub mod azure_devops_command;
pub mod azure_devops_rest_command;
pub mod group;
pub mod license_entitlement;
pub mod project;
pub mod repo;
pub mod service_endpoint;
pub mod team;
pub mod test;
pub mod work_item_query;
pub mod user;

use crate::cli::azure_devops::azure_devops_command::AzureDevOpsCommand;
use clap::Args;
use eyre::Result;

/// Arguments for Azure DevOps-specific operations.
#[derive(Args, Debug, Clone)]
pub struct AzureDevOpsArgs {
    #[command(subcommand)]
    pub command: AzureDevOpsCommand,
}

impl AzureDevOpsArgs {
    pub async fn invoke(self) -> Result<()> {
        self.command.invoke().await
    }
}
