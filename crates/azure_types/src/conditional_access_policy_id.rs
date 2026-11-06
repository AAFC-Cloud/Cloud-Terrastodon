use arbitrary::Arbitrary;
use facet::Facet;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Arbitrary, Facet)]
#[facet(json::proxy = String)]
pub struct ConditionalAccessPolicyId(uuid::Uuid);

crate::impl_uuid_newtype!(ConditionalAccessPolicyId);

cloud_terrastodon_registry::register_thing!(ConditionalAccessPolicyId);
cloud_terrastodon_registry::register_arbitrary!(ConditionalAccessPolicyId);
