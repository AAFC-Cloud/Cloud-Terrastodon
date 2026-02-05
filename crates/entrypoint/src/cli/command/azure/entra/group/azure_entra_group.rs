use super::list::AzureEntraGroupListArgs;
use super::member::AzureEntraGroupMemberArgs;
use super::show::AzureEntraGroupShowArgs;
use clap::Subcommand;
use eyre::Result;

/// Entra group top-level subcommands.
#[derive(Subcommand, Debug, Clone)]
pub enum AzureEntraGroupCommand {
    /// List Entra groups.
    List(AzureEntraGroupListArgs),
    /// Show an Entra group by id.
    Show(AzureEntraGroupShowArgs),
    /// Operations on group members.
    Member(AzureEntraGroupMemberArgs),
}

impl AzureEntraGroupCommand {
    pub async fn invoke(self) -> Result<()> {
        match self {
            AzureEntraGroupCommand::List(args) => args.invoke().await?,
            AzureEntraGroupCommand::Show(args) => args.invoke().await?,
            AzureEntraGroupCommand::Member(args) => args.invoke().await?,
        }
        Ok(())
    }
}
