use crate::impl_uuid_traits;
use crate::prelude::GroupId;
use crate::prelude::ServicePrincipalId;
use crate::prelude::UserId;
use crate::prelude::UuidWrapper;
use serde::de::Error;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use std::ops::Deref;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum PrincipalId {
    UserId(UserId),
    GroupId(GroupId),
    ServicePrincipalId(ServicePrincipalId),
    Unknown(Uuid),
}
impl UuidWrapper for PrincipalId {
    fn new(uuid: Uuid) -> Self {
        Self::Unknown(uuid)
    }

    fn as_ref(&self) -> &Uuid {
        match self {
            PrincipalId::UserId(inner) => &inner.as_ref(),
            PrincipalId::GroupId(inner) => &inner.as_ref(),
            PrincipalId::ServicePrincipalId(inner) => &inner.as_ref(),
            PrincipalId::Unknown(inner) => &inner.as_ref(),
        }
    }
}
impl_uuid_traits!(PrincipalId);

impl From<Uuid> for PrincipalId {
    fn from(value: Uuid) -> Self {
        Self::Unknown(value)
    }
}
impl From<UserId> for PrincipalId {
    fn from(value: UserId) -> Self {
        Self::UserId(value)
    }
}
impl From<GroupId> for PrincipalId {
    fn from(value: GroupId) -> Self {
        Self::GroupId(value)
    }
}
impl From<ServicePrincipalId> for PrincipalId {
    fn from(value: ServicePrincipalId) -> Self {
        Self::ServicePrincipalId(value)
    }
}