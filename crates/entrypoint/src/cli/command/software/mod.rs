pub mod software_list;

use crate::cli::software::software_list::SoftwareListArgs;
use eyre::Result;
use teamy_cancellation::CancellationToken;

/// Software discovery commands
#[derive(facet::Facet, Debug, Clone)]
pub struct SoftwareArgs {
    #[facet(figue::subcommand)]
    pub command: SoftwareCommand,
}

#[derive(facet::Facet, Debug, Clone)]
#[repr(u8)]
pub enum SoftwareCommand {
    /// List known software patterns and their match counts.
    List(SoftwareListArgs),
}

impl SoftwareArgs {
    pub async fn invoke(self, cancellation_token: &CancellationToken) -> Result<()> {
        self.command.invoke(cancellation_token).await
    }
}

impl SoftwareCommand {
    pub async fn invoke(self, cancellation_token: &CancellationToken) -> Result<()> {
        match self {
            Self::List(args) => args.invoke(cancellation_token).await,
        }
    }
}
