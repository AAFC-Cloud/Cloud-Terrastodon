use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_bmc_helix::BmcHelixServerStore;
use eyre::Result;
use facet::Facet;

#[derive(Debug, Clone, Facet)]
pub struct BmcHelixServerListArgs {}

impl BmcHelixServerListArgs {
    pub fn invoke(self) -> Result<CliOutput> {
        Ok(CliOutput::facet(BmcHelixServerStore::configured().load()?))
    }
}
