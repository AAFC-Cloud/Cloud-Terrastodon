use cloud_terrastodon_auth_types::CloudTerrastodonAccountId;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountSession;
use cloud_terrastodon_bmc_helix_types::BmcHelixServerUrl;
use facet::Facet;

/// Lifecycle of a Helix account's JWT on the server that issued it.
/// The generation identifies a separate secret-store entry for that token.
#[derive(Debug, Clone, Eq, PartialEq, Facet)]
pub struct BmcHelixAccountSession {
    pub account_id: CloudTerrastodonAccountId,
    pub server: BmcHelixServerUrl,
    pub session: CloudTerrastodonAccountSession,
}
