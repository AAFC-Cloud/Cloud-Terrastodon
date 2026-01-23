#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct EntraGroupId(pub uuid::Uuid);

crate::impl_uuid_newtype!(EntraGroupId);
