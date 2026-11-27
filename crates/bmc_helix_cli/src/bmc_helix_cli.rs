use crate::BmcHelixServerArgs;
use crate::BmcHelixTicketArgs;
use crate::BmcHelixWorkOrderArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

#[derive(Debug, Facet)]
pub struct BmcHelixArgs {
    #[facet(figue::subcommand)]
    pub command: BmcHelixCommand,
}

#[derive(Debug, Facet)]
#[repr(u8)]
pub enum BmcHelixCommand {
    /// Manage the BMC Helix servers tracked on this device.
    Server(BmcHelixServerArgs),
    /// Browse BMC Helix work orders.
    #[facet(figue::alias = "wo")]
    WorkOrder(BmcHelixWorkOrderArgs),
    /// Browse the Shared Ticket Console.
    Ticket(BmcHelixTicketArgs),
}

impl BmcHelixArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        match self.command {
            BmcHelixCommand::Server(args) => args.invoke(),
            BmcHelixCommand::WorkOrder(args) => Box::pin(args.invoke(executor)).await,
            BmcHelixCommand::Ticket(args) => Box::pin(args.invoke(executor)).await,
        }
    }
}
