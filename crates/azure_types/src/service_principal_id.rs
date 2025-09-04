#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct ServicePrincipalId(uuid::Uuid);

crate::impl_uuid_newtype!(ServicePrincipalId);
