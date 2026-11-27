use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_bmc_helix::BmcHelixServerStore;
use cloud_terrastodon_bmc_helix_types::BmcHelixServerName;
use eyre::Result;
use facet::Facet;

#[derive(Debug, Clone, Facet)]
pub struct BmcHelixServerRemoveArgs {
    /// Tracked server name.
    #[facet(figue::positional)]
    pub name: BmcHelixServerName,
}

impl BmcHelixServerRemoveArgs {
    pub fn invoke(self) -> Result<CliOutput> {
        Ok(CliOutput::facet(
            BmcHelixServerStore::configured().remove(&self.name)?,
        ))
    }
}
