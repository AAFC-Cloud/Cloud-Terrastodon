use super::azure_entra_group_list_cli::AzureEntraGroupListArgs;
use super::azure_entra_group_show_cli::AzureEntraGroupShowArgs;
use super::member::AzureEntraGroupMemberArgs;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;

/// Entra group top-level subcommands.
#[derive(facet::Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureEntraGroupCommand {
    /// List Entra groups.
    List(AzureEntraGroupListArgs),
    /// Show an Entra group by id.
    Show(AzureEntraGroupShowArgs),
    /// Operations on group members.
    Member(AzureEntraGroupMemberArgs),
}

impl AzureEntraGroupCommand {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<()> {
        match self {
            AzureEntraGroupCommand::List(args) => args.invoke(auth_context).await?,
            AzureEntraGroupCommand::Show(args) => args.invoke(auth_context).await?,
            AzureEntraGroupCommand::Member(args) => args.invoke(auth_context).await?,
        }
        Ok(())
    }
}
