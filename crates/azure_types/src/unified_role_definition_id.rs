#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, arbitrary::Arbitrary, facet::Facet)]
#[facet(json::proxy = String)]
pub struct UnifiedRoleDefinitionId(uuid::Uuid);

crate::impl_uuid_newtype!(UnifiedRoleDefinitionId);
