use arbitrary::Arbitrary;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Arbitrary)]
pub struct EntraApplicationRegistrationId(uuid::Uuid);

crate::impl_uuid_newtype!(EntraApplicationRegistrationId);
