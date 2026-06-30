pub mod gitea_command;
pub mod org;
pub mod repo;
pub mod tenant;
pub mod user;

use crate::cli::gitea::gitea_command::GiteaCommand;
use eyre::Result;

#[derive(facet::Facet, Debug, Clone)]
pub struct GiteaArgs {
    #[facet(figue::subcommand)]
    pub command: GiteaCommand,
}

impl GiteaArgs {
    pub async fn invoke(self) -> Result<()> {
        self.command.invoke().await
    }
}
