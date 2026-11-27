use super::BmcHelixTicketListArgs;
use super::BmcHelixTicketShowArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

#[derive(Debug, Facet)]
pub struct BmcHelixTicketArgs {
    #[facet(figue::subcommand)]
    pub command: BmcHelixTicketCommand,
}

#[derive(Debug, Facet)]
#[repr(u8)]
pub enum BmcHelixTicketCommand {
    /// List open, assigned tickets for your support groups.
    List(BmcHelixTicketListArgs),
    /// Show a ticket by record ID among the Shared Ticket Console results.
    Show(BmcHelixTicketShowArgs),
}

impl BmcHelixTicketArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        match self.command {
            BmcHelixTicketCommand::List(args) => Box::pin(args.invoke(executor)).await,
            BmcHelixTicketCommand::Show(args) => Box::pin(args.invoke(executor)).await,
        }
    }
}
