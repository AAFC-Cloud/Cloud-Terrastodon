#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, facet::Facet)]
#[facet(json::proxy = String)]
pub struct ConditionalAccessPolicyId(uuid::Uuid);

crate::impl_uuid_newtype!(ConditionalAccessPolicyId);
