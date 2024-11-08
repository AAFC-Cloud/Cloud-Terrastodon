use crate::impl_uuid_traits;
use crate::prelude::Group;
use crate::prelude::GroupId;
use crate::prelude::ServicePrincipal;
use crate::prelude::ServicePrincipalId;
use crate::prelude::User;
use crate::prelude::UserId;
use crate::prelude::UuidWrapper;
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

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Principal {
    User(User),
    Group(Group),
    ServicePrincipal(ServicePrincipal),
}
impl From<User> for Principal {
    fn from(value: User) -> Self {
        Self::User(value)
    }
}
impl From<Group> for Principal {
    fn from(value: Group) -> Self {
        Self::Group(value)
    }
}
impl From<ServicePrincipal> for Principal {
    fn from(value: ServicePrincipal) -> Self {
        Self::ServicePrincipal(value)
    }
}