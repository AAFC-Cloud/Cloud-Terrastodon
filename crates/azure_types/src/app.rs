#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct AppId(uuid::Uuid);

crate::impl_uuid_newtype!(AppId);
