pub mod azure_entra_group_member_add;
pub mod azure_entra_group_member_remove;

pub use azure_entra_group_member_add::AzureEntraGroupMemberAddArgs;
pub use azure_entra_group_member_remove::AzureEntraGroupMemberRemoveArgs;
use clap::Args;
use eyre::Result;

/// Group member operations (add/remove)
#[derive(Args, Debug, Clone)]
pub struct AzureEntraGroupMemberArgs {
    #[command(subcommand)]
    pub command: AzureEntraGroupMemberCommand,
}

#[derive(clap::Subcommand, Debug, Clone)]
pub enum AzureEntraGroupMemberCommand {
    /// Add a member to a group.
    Add(AzureEntraGroupMemberAddArgs),
    /// Remove a member from a group.
    Remove(AzureEntraGroupMemberRemoveArgs),
}

impl AzureEntraGroupMemberArgs {
    pub async fn invoke(self) -> Result<()> {
        match self.command {
            AzureEntraGroupMemberCommand::Add(a) => a.invoke().await?,
            AzureEntraGroupMemberCommand::Remove(a) => a.invoke().await?,
        }
        Ok(())
    }
}
