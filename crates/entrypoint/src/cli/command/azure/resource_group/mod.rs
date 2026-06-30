pub mod azure_resource_group;
pub mod azure_resource_group_browse;
pub mod azure_resource_group_list;

pub use azure_resource_group::AzureResourceGroupCommand;
pub use azure_resource_group_browse::AzureResourceGroupBrowseArgs;
pub use azure_resource_group_list::AzureResourceGroupListArgs;
use eyre::Result;

/// Manage Azure resource groups.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureResourceGroupArgs {
    #[facet(figue::subcommand)]
    pub command: AzureResourceGroupCommand,
}

impl AzureResourceGroupArgs {
    pub async fn invoke(self) -> Result<()> {
        self.command.invoke().await
    }
}
