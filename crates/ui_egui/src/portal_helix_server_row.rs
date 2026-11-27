use crate::portal_helix_server_view::PortalHelixServerView;
use crate::portal_tab_target::PortalTabTarget;
use cloud_terrastodon_bmc_helix_types::BmcHelixServerName;
use cloud_terrastodon_bmc_helix_types::BmcHelixServerUrl;
use facet::Facet;

/// Searchable server registration in the Helix inventory.
#[derive(Facet)]
pub(crate) struct PortalHelixServerRow {
    pub name: BmcHelixServerName,
    pub url: BmcHelixServerUrl,
    pub default: &'static str,
    pub target: PortalTabTarget,
}

impl PortalHelixServerRow {
    pub fn new(name: &BmcHelixServerName, url: &BmcHelixServerUrl, is_default: bool) -> Self {
        Self {
            name: name.clone(),
            url: url.clone(),
            default: if is_default { "Yes" } else { "No" },
            target: PortalHelixServerView::Overview.target(name),
        }
    }
}
