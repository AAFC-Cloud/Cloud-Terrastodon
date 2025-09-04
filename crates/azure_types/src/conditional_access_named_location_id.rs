#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct ConditionalAccessNamedLocationId(uuid::Uuid);

crate::impl_uuid_newtype!(ConditionalAccessNamedLocationId);
