use crate::impl_uuid_traits;
use crate::prelude::UuidWrapper;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct UserId(Uuid);
impl UuidWrapper for UserId {
    fn new(uuid: Uuid) -> Self {
        Self(uuid)
    }

    fn as_ref(&self) -> &Uuid {
        &self.0
    }
}
impl_uuid_traits!(UserId);
