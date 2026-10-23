pub mod azure_entra_group_member_add_cli;
pub mod azure_entra_group_member_list_cli;
pub mod azure_entra_group_member_remove_cli;

pub use azure_entra_group_member_add_cli::AzureEntraGroupMemberAddArgs;
pub use azure_entra_group_member_list_cli::AzureEntraGroupMemberListArgs;
pub use azure_entra_group_member_remove_cli::AzureEntraGroupMemberRemoveArgs;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_credentials::AuthContext;
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
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        cancellation: &CancellationScope,
    ) -> Result<()> {
        match self.command {
            AzureEntraGroupMemberCommand::Add(a) => a.invoke(cancellation).await?,
            AzureEntraGroupMemberCommand::List(a) => a.invoke(auth_context, cancellation).await?,
            AzureEntraGroupMemberCommand::Remove(a) => a.invoke(cancellation).await?,
        }
        Ok(())
    }
}
