use arbitrary::Arbitrary;
use facet::Facet;
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Arbitrary, Facet)]
#[facet(json::proxy = String)]
pub struct UnifiedRoleDefinitionId(uuid::Uuid);

crate::impl_uuid_newtype!(UnifiedRoleDefinitionId);
