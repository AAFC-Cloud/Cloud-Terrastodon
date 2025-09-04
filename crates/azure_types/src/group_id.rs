#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct GroupId(pub uuid::Uuid);

crate::impl_uuid_newtype!(GroupId);
