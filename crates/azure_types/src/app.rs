use arbitrary::Arbitrary;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Arbitrary)]
pub struct AppId(uuid::Uuid);

crate::impl_uuid_newtype!(AppId);
