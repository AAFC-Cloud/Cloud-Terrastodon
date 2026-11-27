use super::bmc_helix_server_add_cli::BmcHelixServerAddArgs;
use super::bmc_helix_server_default_cli::BmcHelixServerDefaultArgs;
use super::bmc_helix_server_list_cli::BmcHelixServerListArgs;
use super::bmc_helix_server_remove_cli::BmcHelixServerRemoveArgs;
use super::bmc_helix_server_set_innovation_suite_url_cli::BmcHelixServerSetInnovationSuiteUrlArgs;
use super::bmc_helix_server_show_cli::BmcHelixServerShowArgs;
use cloud_terrastodon_app::CliOutput;
use eyre::Result;
use facet::Facet;

#[derive(Debug, Facet)]
pub struct BmcHelixServerArgs {
    #[facet(figue::subcommand)]
    pub command: BmcHelixServerCommand,
}

#[derive(Debug, Facet)]
#[repr(u8)]
pub enum BmcHelixServerCommand {
    /// List tracked servers and the configured default.
    List(BmcHelixServerListArgs),
    /// Show a tracked server.
    Show(BmcHelixServerShowArgs),
    /// Add a server to the local catalogue.
    Add(BmcHelixServerAddArgs),
    /// Remove a server from the local catalogue.
    Remove(BmcHelixServerRemoveArgs),
    /// Set or clear the Innovation Suite origin for a tracked server.
    SetInnovationSuiteUrl(BmcHelixServerSetInnovationSuiteUrlArgs),
    /// Set or clear the default server.
    Default(BmcHelixServerDefaultArgs),
}

impl BmcHelixServerArgs {
    pub fn invoke(self) -> Result<CliOutput> {
        match self.command {
            BmcHelixServerCommand::List(args) => args.invoke(),
            BmcHelixServerCommand::Show(args) => args.invoke(),
            BmcHelixServerCommand::Add(args) => args.invoke(),
            BmcHelixServerCommand::Remove(args) => args.invoke(),
            BmcHelixServerCommand::SetInnovationSuiteUrl(args) => args.invoke(),
            BmcHelixServerCommand::Default(args) => args.invoke(),
        }
    }
}
