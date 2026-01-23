#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct EntraServicePrincipalId(uuid::Uuid);

crate::impl_uuid_newtype!(EntraServicePrincipalId);
