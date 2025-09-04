#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct ConditionalAccessPolicyId(uuid::Uuid);

crate::impl_uuid_newtype!(ConditionalAccessPolicyId);
