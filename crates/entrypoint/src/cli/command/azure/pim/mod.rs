pub mod azure_pim_activate;
pub mod azure_pim_setup;

pub use azure_pim_activate::AzurePimActivateArgs;
pub use azure_pim_setup::AzurePimSetupArgs;
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
    /// Discover and configure the Cloud Terrastodon PIM app registration.
    Setup(AzurePimSetupArgs),
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
            AzurePimCommand::Setup(args) => args.invoke().await,
        }
    }
}
