use crate::impl_uuid_traits;
use crate::prelude::Group;
use crate::prelude::GroupId;
use crate::prelude::ServicePrincipal;
use crate::prelude::ServicePrincipalId;
use crate::prelude::User;
use crate::prelude::UserId;
use crate::prelude::UuidWrapper;
use serde::Deserialize;
use serde::Serialize;
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

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "@odata.type")]
pub enum Principal {
    #[serde(rename = "#microsoft.graph.user")]
    User(User),
    #[serde(rename = "#microsoft.graph.group")]
    Group(Group),
    #[serde(rename = "#microsoft.graph.servicePrincipal")]
    ServicePrincipal(Box<ServicePrincipal>),
    // #[serde(rename = "#microsoft.graph.device")]
    // Device(Value),
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
        Self::ServicePrincipal(Box::new(value))
    }
}
impl AsRef<Uuid> for Principal {
    fn as_ref(&self) -> &Uuid {
        match self {
            Principal::User(user) => &user.id,
            Principal::Group(group) => &group.id,
            Principal::ServicePrincipal(service_principal) => &service_principal.id,
        }
    }
}
impl Principal {
    pub fn display_name(&self) -> &str {
        match self {
            Principal::User(user) => &user.user_principal_name,
            Principal::Group(group) => &group.display_name,
            Principal::ServicePrincipal(service_principal) => &service_principal.display_name,
        }
    }
    pub fn id(&self) -> PrincipalId {
        match self {
            Principal::User(user) => PrincipalId::UserId(user.id),
            Principal::Group(group) => PrincipalId::GroupId(group.id),
            Principal::ServicePrincipal(service_principal) => {
                PrincipalId::ServicePrincipalId(service_principal.id)
            }
        }
    }
}
impl std::fmt::Display for Principal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "({}) {} ({})",
            match self {
                Principal::Group(..) => "group",
                Principal::ServicePrincipal(..) => "service principal",
                Principal::User(..) => "user",
            },
            self.display_name(),
            self.id()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn it_works() -> eyre::Result<()> {
        let user: User = User {
            business_phones: vec![],
            display_name: "User, Fake".to_string(),
            given_name: Some("User".to_string()),
            id: UserId::new(Uuid::nil()),
            job_title: None,
            mail: None,
            mobile_phone: None,
            office_location: None,
            preferred_language: None,
            surname: Some("Fake".to_string()),
            user_principal_name: "fake.user@example.com".to_string(),
        };
        let principal = Principal::from(user);
        let encoded = serde_json::to_string_pretty(&principal)?;
        println!("Encoded:\n{encoded}");
        let decoded: Value = serde_json::from_str(&encoded)?;
        assert_eq!(
            decoded.get("@odata.type").unwrap().as_str().unwrap(),
            "#microsoft.graph.user"
        );
        let decoded_principal = serde_json::from_value::<Principal>(decoded)?;
        assert_eq!(principal, decoded_principal);
        Ok(())
    }
}
