use arbitrary::Arbitrary;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Arbitrary)]
pub struct UserId(uuid::Uuid);

crate::impl_uuid_newtype!(UserId);
