use cloud_terrastodon_bmc_helix_types::BmcHelixServer;
use cloud_terrastodon_bmc_helix_types::BmcHelixServerName;
use cloud_terrastodon_bmc_helix_types::BmcHelixServerUrl;
use eyre::Result;
use eyre::bail;
use eyre::ensure;
use facet::Facet;
use std::collections::BTreeMap;
use std::collections::HashMap;

/// Saved Helix endpoints and the explicitly selected default endpoint.
#[derive(Debug, Clone, Default, PartialEq, Eq, Facet)]
#[facet(deny_unknown_fields)]
pub struct BmcHelixServerCatalog {
    pub servers: BTreeMap<BmcHelixServerName, BmcHelixServerUrl>,
    pub default: Option<BmcHelixServerName>,
    /// Additional origins explicitly authorized to receive the server's JWT.
    #[facet(default)]
    pub innovation_suite_urls: BTreeMap<BmcHelixServerName, BmcHelixServerUrl>,
}

impl BmcHelixServerCatalog {
    /// Resolves an explicit name or the configured default.
    pub fn get(&self, name: Option<&BmcHelixServerName>) -> Result<BmcHelixServer> {
        let Some(name) = name.or(self.default.as_ref()) else {
            bail!("Select a Helix server or configure a default server");
        };
        let Some(url) = self.servers.get(name) else {
            bail!("Helix server {name:?} is not registered");
        };
        Ok(BmcHelixServer {
            name: name.clone(),
            url: url.clone(),
            innovation_suite_url: self.innovation_suite_url(name).cloned(),
        })
    }

    pub fn innovation_suite_url(&self, name: &BmcHelixServerName) -> Option<&BmcHelixServerUrl> {
        self.innovation_suite_urls.get(name)
    }

    /// Find the explicitly authorized Innovation Suite origin for an admitted
    /// AR System origin. Unconfigured servers use the issuing origin.
    pub fn innovation_suite_url_for_origin(
        &self,
        origin: &BmcHelixServerUrl,
    ) -> Option<&BmcHelixServerUrl> {
        self.servers
            .iter()
            .find(|(_, url)| *url == origin)
            .and_then(|(name, _)| self.innovation_suite_url(name))
    }

    pub(crate) fn validate(&self) -> Result<()> {
        if let Some(default) = &self.default {
            ensure!(
                self.servers.contains_key(default),
                "The default Helix server {default:?} is not registered"
            );
        }
        let mut names_by_origin = HashMap::with_capacity(self.servers.len());
        for (name, origin) in &self.servers {
            if let Some(existing) = names_by_origin.insert(origin, name) {
                bail!("Helix server {name:?} has the same origin as {existing:?}");
            }
        }
        ensure!(
            self.innovation_suite_urls
                .keys()
                .all(|name| self.servers.contains_key(name)),
            "Innovation Suite URLs must belong to a registered Helix server"
        );
        Ok(())
    }
}

cloud_terrastodon_registry::register_thing!(BmcHelixServerCatalog);
