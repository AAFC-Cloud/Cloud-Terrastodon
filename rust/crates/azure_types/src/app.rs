use crate::impl_uuid_traits;
use crate::prelude::UuidWrapper;
use uuid::Uuid;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct AppId(Uuid);
impl UuidWrapper for AppId {
    fn new(uuid: Uuid) -> Self {
        Self(uuid)
    }

    fn as_ref(&self) -> &Uuid {
        &self.0
    }
}
impl_uuid_traits!(AppId);
