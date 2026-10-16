pub mod gitea_command_cli;
pub mod org;
pub mod repo;
pub mod tenant;
pub mod user;

use crate::cli::gitea::gitea_command_cli::GiteaCommand;
use cloud_terrastodon_command::CancellationScope;
use eyre::Result;

#[derive(facet::Facet, Debug, Clone)]
pub struct GiteaArgs {
    #[facet(figue::subcommand)]
    pub command: GiteaCommand,
}

impl GiteaArgs {
    pub async fn invoke(
        self,
        cancellation: &CancellationScope,
    ) -> Result<()> {
        self.command.invoke(cancellation).await
    }
}
