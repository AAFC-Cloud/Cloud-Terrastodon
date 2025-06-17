use crate::impl_uuid_traits;
use crate::prelude::UuidWrapper;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct ConditionalAccessPolicyId(Uuid);
impl UuidWrapper for ConditionalAccessPolicyId {
    fn new(uuid: Uuid) -> Self {
        Self(uuid)
    }

    fn as_ref(&self) -> &Uuid {
        &self.0
    }
}
impl_uuid_traits!(ConditionalAccessPolicyId);
