use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_bmc_helix::BmcHelixServerStore;
use cloud_terrastodon_bmc_helix_types::BmcHelixServerName;
use cloud_terrastodon_bmc_helix_types::BmcHelixServerUrl;
use eyre::Result;
use facet::Facet;

#[derive(Debug, Clone, Facet)]
pub struct BmcHelixServerSetInnovationSuiteUrlArgs {
    /// Tracked server name.
    #[facet(figue::positional)]
    pub server: BmcHelixServerName,
    /// Innovation Suite HTTPS origin. Omit to use the AR System origin.
    #[facet(figue::positional)]
    pub url: Option<BmcHelixServerUrl>,
}

impl BmcHelixServerSetInnovationSuiteUrlArgs {
    pub fn invoke(self) -> Result<CliOutput> {
        let server =
            BmcHelixServerStore::configured().set_innovation_suite_url(&self.server, self.url)?;
        Ok(CliOutput::facet(server))
    }
}
