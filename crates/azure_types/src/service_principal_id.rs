use arbitrary::Arbitrary;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Arbitrary)]
pub struct EntraServicePrincipalId(uuid::Uuid);

crate::impl_uuid_newtype!(EntraServicePrincipalId);
