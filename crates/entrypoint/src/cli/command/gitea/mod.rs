pub mod gitea_command;
pub mod org;
pub mod repo;
pub mod tenant;
pub mod user;

use crate::cli::gitea::gitea_command::GiteaCommand;
use clap::Args;
use eyre::Result;

#[derive(Args, Debug, Clone)]
pub struct GiteaArgs {
    #[command(subcommand)]
    pub command: GiteaCommand,
}

impl GiteaArgs {
    pub async fn invoke(self) -> Result<()> {
        self.command.invoke().await
    }
}
