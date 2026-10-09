pub mod agent;
pub mod agent_package;
pub mod audit_cli;
pub mod azure_devops_command_cli;
pub mod azure_devops_rest_command_cli;
pub mod build;
pub mod group;
pub mod license_entitlement;
pub mod project;
pub mod repo;
pub mod service_endpoint;
pub mod team;
pub mod test;
pub mod work_item;

use crate::cli::azure_devops::azure_devops_command_cli::AzureDevOpsCommand;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;

/// Arguments for Azure DevOps-specific operations.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureDevOpsArgs {
    #[facet(figue::subcommand)]
    pub command: AzureDevOpsCommand,
}

impl AzureDevOpsArgs {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<CliOutput> {
        self.command.invoke(auth_context).await
    }
}
