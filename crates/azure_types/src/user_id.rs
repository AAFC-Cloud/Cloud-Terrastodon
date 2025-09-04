#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct UserId(uuid::Uuid);

crate::impl_uuid_newtype!(UserId);
