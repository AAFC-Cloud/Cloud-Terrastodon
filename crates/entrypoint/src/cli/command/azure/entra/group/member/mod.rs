pub mod azure_entra_group_member_add;
pub mod azure_entra_group_member_list;
pub mod azure_entra_group_member_remove;

pub use azure_entra_group_member_add::AzureEntraGroupMemberAddArgs;
pub use azure_entra_group_member_list::AzureEntraGroupMemberListArgs;
pub use azure_entra_group_member_remove::AzureEntraGroupMemberRemoveArgs;
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
