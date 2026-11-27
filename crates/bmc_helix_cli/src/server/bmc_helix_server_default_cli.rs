use super::bmc_helix_server_default_clear_cli::BmcHelixServerDefaultClearArgs;
use super::bmc_helix_server_default_set_cli::BmcHelixServerDefaultSetArgs;
use cloud_terrastodon_app::CliOutput;
use eyre::Result;
use facet::Facet;

#[derive(Debug, Facet)]
pub struct BmcHelixServerDefaultArgs {
    #[facet(figue::subcommand)]
    pub command: BmcHelixServerDefaultCommand,
}

#[derive(Debug, Facet)]
#[repr(u8)]
pub enum BmcHelixServerDefaultCommand {
    /// Set a tracked server as the default.
    Set(BmcHelixServerDefaultSetArgs),
    /// Clear the default server.
    Clear(BmcHelixServerDefaultClearArgs),
}

impl BmcHelixServerDefaultArgs {
    pub fn invoke(self) -> Result<CliOutput> {
        match self.command {
            BmcHelixServerDefaultCommand::Set(args) => args.invoke(),
            BmcHelixServerDefaultCommand::Clear(args) => args.invoke(),
        }
    }
}
