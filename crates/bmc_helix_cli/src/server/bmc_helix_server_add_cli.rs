use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_bmc_helix::BmcHelixServerStore;
use cloud_terrastodon_bmc_helix_types::BmcHelixServer;
use cloud_terrastodon_bmc_helix_types::BmcHelixServerName;
use cloud_terrastodon_bmc_helix_types::BmcHelixServerUrl;
use eyre::Result;
use facet::Facet;

#[derive(Debug, Clone, Facet)]
pub struct BmcHelixServerAddArgs {
    /// Local name for this server.
    #[facet(figue::named)]
    pub name: BmcHelixServerName,
    /// Base HTTPS URL for the AR System and sign-in.
    #[facet(figue::named)]
    pub url: BmcHelixServerUrl,
    /// Innovation Suite HTTPS origin when it differs from the AR System origin.
    #[facet(figue::named)]
    pub innovation_suite_url: Option<BmcHelixServerUrl>,
}

impl BmcHelixServerAddArgs {
    pub fn invoke(self) -> Result<CliOutput> {
        Ok(CliOutput::facet(BmcHelixServerStore::configured().add(
            BmcHelixServer {
                name: self.name,
                url: self.url,
                innovation_suite_url: self.innovation_suite_url,
            },
        )?))
    }
}
