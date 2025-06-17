use crate::impl_uuid_traits;
use crate::prelude::UuidWrapper;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct ConditionalAccessNamedLocationId(Uuid);
impl UuidWrapper for ConditionalAccessNamedLocationId {
    fn new(uuid: Uuid) -> Self {
        Self(uuid)
    }

    fn as_ref(&self) -> &Uuid {
        &self.0
    }
}
impl_uuid_traits!(ConditionalAccessNamedLocationId);
