use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_bmc_helix::BmcHelixServerStore;
use cloud_terrastodon_bmc_helix_types::BmcHelixServerName;
use eyre::Result;
use facet::Facet;

#[derive(Debug, Clone, Facet)]
pub struct BmcHelixServerShowArgs {
    /// Tracked server name. Omit to show the configured default.
    #[facet(figue::positional)]
    pub name: Option<BmcHelixServerName>,
}

impl BmcHelixServerShowArgs {
    pub fn invoke(self) -> Result<CliOutput> {
        Ok(CliOutput::facet(
            BmcHelixServerStore::configured().get(self.name.as_ref())?,
        ))
    }
}
