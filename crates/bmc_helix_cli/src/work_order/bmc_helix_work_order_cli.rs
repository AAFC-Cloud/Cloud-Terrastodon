use super::BmcHelixWorkOrderListArgs;
use super::BmcHelixWorkOrderShowArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

#[derive(Debug, Facet)]
pub struct BmcHelixWorkOrderArgs {
    #[facet(figue::subcommand)]
    pub command: BmcHelixWorkOrderCommand,
}

#[derive(Debug, Facet)]
#[repr(u8)]
pub enum BmcHelixWorkOrderCommand {
    /// List work orders matching an optional BMC qualification.
    List(BmcHelixWorkOrderListArgs),
    /// Show a work order by its ID.
    Show(BmcHelixWorkOrderShowArgs),
}

impl BmcHelixWorkOrderArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        match self.command {
            BmcHelixWorkOrderCommand::List(args) => Box::pin(args.invoke(executor)).await,
            BmcHelixWorkOrderCommand::Show(args) => Box::pin(args.invoke(executor)).await,
        }
    }
}
