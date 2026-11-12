use arbitrary::Arbitrary;
use cloud_terrastodon_azure_types::uuid::Uuid;
use facet::Facet;

/// Stable local identity of a Cloud Terrastodon account, shared by its credential mechanisms.
///
/// Aliases can change; this identity must never be reassigned to another principal.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Arbitrary, Facet)]
#[facet(json::proxy = String)]
pub struct CloudTerrastodonAccountId(Uuid);

cloud_terrastodon_azure_types::impl_uuid_newtype!(CloudTerrastodonAccountId);
cloud_terrastodon_registry::register_thing!(CloudTerrastodonAccountId);
cloud_terrastodon_registry::register_arbitrary!(CloudTerrastodonAccountId);
