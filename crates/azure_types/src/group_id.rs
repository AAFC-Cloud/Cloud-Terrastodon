use crate::impl_uuid_traits;
use crate::prelude::UuidWrapper;
use eyre::Result;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct GroupId(pub Uuid);
impl UuidWrapper for GroupId {
    fn new(uuid: Uuid) -> Self {
        Self(uuid)
    }

    fn as_ref(&self) -> &Uuid {
        &self.0
    }
}
impl_uuid_traits!(GroupId);