pub mod azure_pim_activate;

pub use azure_pim_activate::AzurePimActivateArgs;
use eyre::Result;

/// Arguments for Azure Privileged Identity Management operations.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzurePimArgs {
    #[facet(figue::subcommand)]
    pub command: AzurePimCommand,
}

/// Subcommands available under `cloud_terrastodon az pim`.
#[derive(facet::Facet, Debug, Clone)]
#[repr(u8)]
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
