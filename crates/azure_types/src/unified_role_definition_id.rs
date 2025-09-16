#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct UnifiedRoleDefinitionId(uuid::Uuid);

crate::impl_uuid_newtype!(UnifiedRoleDefinitionId);
