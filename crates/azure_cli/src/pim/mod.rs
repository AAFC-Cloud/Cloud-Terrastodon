use cloud_terrastodon_app::CliOutput;
mod azure_pim_required_resource_access;
mod azure_pim_required_resource_access_entry;
mod azure_pim_setup_client_id_output;
mod azure_pim_setup_output;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use facet::Facet;
pub mod azure_pim_activate_cli;
pub mod azure_pim_setup_cli;

pub use azure_pim_activate_cli::AzurePimActivateArgs;
pub use azure_pim_setup_cli::AzurePimSetupArgs;
use eyre::Result;

/// Arguments for Azure Privileged Identity Management operations.
#[derive(Facet, Debug, Clone)]
pub struct AzurePimArgs {
    #[facet(figue::subcommand)]
    pub command: AzurePimCommand,
}

/// Subcommands available under `cloud_terrastodon az pim`.
#[derive(Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzurePimCommand {
    /// Activate Azure or Entra PIM assignments.
    Activate(AzurePimActivateArgs),
    /// Inspect a PIM app registration and show named-account registration guidance.
    Setup(AzurePimSetupArgs),
}

impl AzurePimArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        self.command.invoke(executor).await
    }
}

impl AzurePimCommand {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        match self {
            AzurePimCommand::Activate(args) => Box::pin(args.invoke(executor)).await,
            AzurePimCommand::Setup(args) => Box::pin(args.invoke(executor)).await,
        }
    }
}
