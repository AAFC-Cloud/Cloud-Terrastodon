use crate::BmcHelixServerName;
use crate::BmcHelixServerUrl;
use arbitrary::Arbitrary;
use facet::Facet;

/// A named Helix endpoint saved in Cloud Terrastodon. Accounts are registered separately.
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, Facet)]
pub struct BmcHelixServer {
    pub name: BmcHelixServerName,
    /// AR System origin where Cloud Terrastodon signs in.
    pub url: BmcHelixServerUrl,
    /// Explicitly trusted host for Innovation Suite requests. When omitted,
    /// Innovation Suite uses the AR System origin.
    pub innovation_suite_url: Option<BmcHelixServerUrl>,
}

impl BmcHelixServer {
    pub fn innovation_suite_origin(&self) -> &BmcHelixServerUrl {
        self.innovation_suite_url.as_ref().unwrap_or(&self.url)
    }
}

cloud_terrastodon_registry::register_thing!(BmcHelixServer);
cloud_terrastodon_registry::register_arbitrary!(BmcHelixServer);
