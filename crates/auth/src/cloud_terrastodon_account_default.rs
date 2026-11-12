use cloud_terrastodon_auth_types::CloudTerrastodonAccountId;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountNamespace;
use facet::Facet;

/// An explicitly configured account default within one namespace.
#[derive(Debug, Clone, Facet)]
pub struct CloudTerrastodonAccountDefault {
    pub namespace: CloudTerrastodonAccountNamespace,
    pub account_id: CloudTerrastodonAccountId,
}
