pub mod app_service;
pub mod application_gateway;
pub mod audit_cli;
pub mod azure_command_cli;
pub mod cognitive_services;
pub mod container_instance;
pub mod entra;
pub mod find_cli;
pub mod network_interface;
pub mod pim;
pub mod policy;
pub mod private_endpoint;
pub mod public_ip;
pub mod resource;
pub mod resource_group;
pub mod role;
pub mod subscription;
pub mod tag;
pub mod tenant;
pub mod vm;

use crate::cli::azure::azure_command_cli::AzureCommand;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;

/// Arguments for Azure-specific operations.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureArgs {
    #[facet(figue::subcommand)]
    pub command: AzureCommand,
}

impl AzureArgs {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<()> {
        self.command.invoke(auth_context).await
    }
}
