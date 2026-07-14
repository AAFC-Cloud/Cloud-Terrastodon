pub mod azure_entra_group_member_add_cli;
pub mod azure_entra_group_member_list_cli;
pub mod azure_entra_group_member_remove_cli;

pub use azure_entra_group_member_add_cli::AzureEntraGroupMemberAddArgs;
pub use azure_entra_group_member_list_cli::AzureEntraGroupMemberListArgs;
pub use azure_entra_group_member_remove_cli::AzureEntraGroupMemberRemoveArgs;
use eyre::Result;

/// Group member operations (add/remove)
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureEntraGroupMemberArgs {
    #[facet(figue::subcommand)]
    pub command: AzureEntraGroupMemberCommand,
}

#[derive(facet::Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureEntraGroupMemberCommand {
    /// Add a member to a group.
    Add(AzureEntraGroupMemberAddArgs),
    /// List the members of a group.
    List(AzureEntraGroupMemberListArgs),
    /// Remove a member from a group.
    Remove(AzureEntraGroupMemberRemoveArgs),
}

impl AzureEntraGroupMemberArgs {
    pub async fn invoke(self) -> Result<()> {
        match self.command {
            AzureEntraGroupMemberCommand::Add(a) => a.invoke().await?,
            AzureEntraGroupMemberCommand::List(a) => a.invoke().await?,
            AzureEntraGroupMemberCommand::Remove(a) => a.invoke().await?,
        }
        Ok(())
    }
}
