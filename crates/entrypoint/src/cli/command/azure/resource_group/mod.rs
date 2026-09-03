use cloud_terrastodon_credentials::AuthContext;
pub mod azure_resource_group_browse_cli;
pub mod azure_resource_group_cli;
pub mod azure_resource_group_list_cli;

pub use azure_resource_group_browse_cli::AzureResourceGroupBrowseArgs;
pub use azure_resource_group_browse_cli::browse_resource_groups;
pub use azure_resource_group_cli::AzureResourceGroupCommand;
pub use azure_resource_group_list_cli::AzureResourceGroupListArgs;
use eyre::Result;

/// Manage Azure resource groups.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureResourceGroupArgs {
    #[facet(figue::subcommand)]
    pub command: AzureResourceGroupCommand,
}

impl AzureResourceGroupArgs {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<()> {
        self.command.invoke(auth_context).await
    }
}
