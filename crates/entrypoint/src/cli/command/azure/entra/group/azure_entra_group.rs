use super::member::AzureEntraGroupMemberArgs;
use clap::Subcommand;
use eyre::Result;

/// Entra group top-level subcommands.
#[derive(Subcommand, Debug, Clone)]
pub enum AzureEntraGroupCommand {
    /// Operations on group members.
    Member(AzureEntraGroupMemberArgs),
}

impl AzureEntraGroupCommand {
    pub async fn invoke(self) -> Result<()> {
        match self {
            AzureEntraGroupCommand::Member(args) => args.invoke().await?,
        }
        Ok(())
    }
}
