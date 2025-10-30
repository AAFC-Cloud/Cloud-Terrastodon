pub mod azure_pim_activate;

pub use azure_pim_activate::AzurePimActivateArgs;
use clap::{Args, Subcommand};
use eyre::Result;

/// Arguments for Azure Privileged Identity Management operations.
#[derive(Args, Debug, Clone)]
pub struct AzurePimArgs {
    #[command(subcommand)]
    pub command: AzurePimCommand,
}

/// Subcommands available under `cloud_terrastodon az pim`.
#[derive(Subcommand, Debug, Clone)]
pub enum AzurePimCommand {
    /// Activate Azure or Entra PIM assignments.
    Activate(AzurePimActivateArgs),
}

impl AzurePimArgs {
    pub async fn invoke(self) -> Result<()> {
        self.command.invoke().await
    }
}

impl AzurePimCommand {
    pub async fn invoke(self) -> Result<()> {
        match self {
            AzurePimCommand::Activate(args) => args.invoke().await,
        }
    }
}
