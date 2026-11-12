use crate::CloudTerrastodonAccountCredential;
use crate::CloudTerrastodonAccountSession;
use facet::Facet;

/// Acquisition configuration and durable lifecycle of one account's mechanism.
#[derive(Debug, Clone, Eq, PartialEq, Facet)]
pub struct CloudTerrastodonCredentialRegistration {
    pub credential: CloudTerrastodonAccountCredential,
    pub session: CloudTerrastodonAccountSession,
}
cloud_terrastodon_registry::register_thing!(CloudTerrastodonCredentialRegistration);
