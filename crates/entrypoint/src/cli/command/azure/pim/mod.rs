pub mod azure_pim_activate_cli;
pub mod azure_pim_setup_cli;

pub use azure_pim_activate_cli::AzurePimActivateArgs;
pub use azure_pim_setup_cli::AzurePimSetupArgs;
use cloud_terrastodon_credentials::AuthContext;
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
    /// Configure the Cloud Terrastodon PIM app registration by client ID or discovery.
    Setup(AzurePimSetupArgs),
}

impl AzurePimArgs {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<()> {
        self.command.invoke(auth_context).await
    }
}

impl AzurePimCommand {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<()> {
        match self {
            AzurePimCommand::Activate(args) => args.invoke(auth_context).await,
            AzurePimCommand::Setup(args) => args.invoke(auth_context).await,
        }
    }
}
