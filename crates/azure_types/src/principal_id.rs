use crate::impl_uuid_traits;
use crate::prelude::GroupId;
use crate::prelude::ServicePrincipalId;
use crate::prelude::UserId;
use crate::prelude::UuidWrapper;
use std::hash::Hash;
use uuid::Uuid;

#[derive(Debug, Clone, Copy)]
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
            PrincipalId::UserId(inner) => inner.as_ref(),
            PrincipalId::GroupId(inner) => inner.as_ref(),
            PrincipalId::ServicePrincipalId(inner) => inner.as_ref(),
            PrincipalId::Unknown(inner) => inner,
        }
    }
}
impl_uuid_traits!(PrincipalId);

// Because the internal uuids should be unique between categories, it's fine to compare between unknown/group/user/etc
impl PartialEq for PrincipalId {
    fn eq(&self, other: &Self) -> bool {
        let left: &Uuid = self;
        let right: &Uuid = other;
        left == right
    }
}
impl Eq for PrincipalId {}
impl Hash for PrincipalId {
    fn hash<H: std::hash::Hasher>(self: &PrincipalId, state: &mut H) {
        let id: &Uuid = self;
        id.hash(state);
    }
}

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
impl From<&Uuid> for PrincipalId {
    fn from(value: &Uuid) -> Self {
        Self::Unknown(*value)
    }
}
impl From<&UserId> for PrincipalId {
    fn from(value: &UserId) -> Self {
        Self::UserId(*value)
    }
}
impl From<&GroupId> for PrincipalId {
    fn from(value: &GroupId) -> Self {
        Self::GroupId(*value)
    }
}
impl From<&ServicePrincipalId> for PrincipalId {
    fn from(value: &ServicePrincipalId) -> Self {
        Self::ServicePrincipalId(*value)
    }
}
impl<T: AsRef<Uuid>> PartialEq<T> for PrincipalId {
    fn eq(&self, other: &T) -> bool {
        let left: &Uuid = self;
        let right: &Uuid = other.as_ref();
        left == right
    }
}
