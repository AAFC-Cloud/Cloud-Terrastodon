pub mod software_list;

use crate::cli::software::software_list::SoftwareListArgs;
use clap::Args;
use clap::Subcommand;
use eyre::Result;

/// Software discovery commands
#[derive(Args, Debug, Clone)]
pub struct SoftwareArgs {
    #[command(subcommand)]
    pub command: SoftwareCommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum SoftwareCommand {
    /// List known software patterns and their match counts.
    List(SoftwareListArgs),
}

impl SoftwareArgs {
    pub async fn invoke(self) -> Result<()> {
        self.command.invoke().await
    }
}

impl SoftwareCommand {
    pub async fn invoke(self) -> Result<()> {
        match self {
            Self::List(args) => args.invoke().await,
        }
    }
}
