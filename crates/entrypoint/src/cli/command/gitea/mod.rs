use facet::Facet;
pub mod gitea_command_cli;
pub mod org;
pub mod repo;
pub mod tenant;
pub mod user;

use crate::cli::gitea::gitea_command_cli::GiteaCommand;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;

#[derive(Facet, Debug, Clone)]
pub struct GiteaArgs {
    #[facet(figue::subcommand)]
    pub command: GiteaCommand,
}

impl GiteaArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<()> {
        self.command.invoke(executor).await
    }
}
