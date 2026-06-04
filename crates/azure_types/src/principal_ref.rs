use crate::EntraGroup;
use crate::EntraServicePrincipal;
use crate::EntraUser;
use crate::Principal;
use crate::PrincipalId;
use crate::PrincipalKind;
use uuid::Uuid;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum PrincipalRef<'a> {
    User(&'a EntraUser),
    Group(&'a EntraGroup),
    ServicePrincipal(&'a EntraServicePrincipal),
    // Device(&'a Value),
}
impl<'a> From<&'a EntraUser> for PrincipalRef<'a> {
    fn from(value: &'a EntraUser) -> Self {
        Self::User(value)
    }
}
impl<'a> From<&'a EntraGroup> for PrincipalRef<'a> {
    fn from(value: &'a EntraGroup) -> Self {
        Self::Group(value)
    }
}
impl<'a> From<&'a EntraServicePrincipal> for PrincipalRef<'a> {
    fn from(value: &'a EntraServicePrincipal) -> Self {
        Self::ServicePrincipal(value)
    }
}
impl<'a> From<&'a Principal> for PrincipalRef<'a> {
    fn from(value: &'a Principal) -> Self {
        match value {
            Principal::User(user) => PrincipalRef::User(user),
            Principal::Group(group) => PrincipalRef::Group(group),
            Principal::ServicePrincipal(service_principal) => {
                PrincipalRef::ServicePrincipal(service_principal)
            }
        }
    }
}
impl AsRef<Uuid> for PrincipalRef<'_> {
    fn as_ref(&self) -> &Uuid {
        match self {
            PrincipalRef::User(user) => &user.id,
            PrincipalRef::Group(group) => &group.id,
            PrincipalRef::ServicePrincipal(service_principal) => &service_principal.id,
        }
    }
}
impl PrincipalRef<'_> {
    pub fn display_name(&self) -> &str {
        match self {
            PrincipalRef::User(user) => &user.user_principal_name,
            PrincipalRef::Group(group) => &group.display_name,
            PrincipalRef::ServicePrincipal(service_principal) => &service_principal.display_name,
        }
    }
    pub fn name(&self) -> &str {
        match self {
            PrincipalRef::User(user) => &user.user_principal_name,
            PrincipalRef::Group(group) => &group.display_name,
            PrincipalRef::ServicePrincipal(service_principal) => &service_principal.display_name,
        }
    }
    pub fn id(&self) -> PrincipalId {
        match self {
            PrincipalRef::User(user) => PrincipalId::UserId(user.id),
            PrincipalRef::Group(group) => PrincipalId::GroupId(group.id),
            PrincipalRef::ServicePrincipal(service_principal) => {
                PrincipalId::ServicePrincipalId(service_principal.id)
            }
        }
    }
    pub fn kind(&self) -> PrincipalKind {
        match self {
            PrincipalRef::User(..) => PrincipalKind::User,
            PrincipalRef::Group(..) => PrincipalKind::Group,
            PrincipalRef::ServicePrincipal(..) => PrincipalKind::ServicePrincipal,
        }
    }
    pub fn as_user(&self) -> Option<&EntraUser> {
        match self {
            PrincipalRef::User(user) => Some(user),
            _ => None,
        }
    }
    pub fn as_group(&self) -> Option<&EntraGroup> {
        match self {
            PrincipalRef::Group(group) => Some(group),
            _ => None,
        }
    }
    pub fn as_service_principal(&self) -> Option<&EntraServicePrincipal> {
        match self {
            PrincipalRef::ServicePrincipal(service_principal) => Some(service_principal),
            _ => None,
        }
    }
}
impl std::fmt::Display for PrincipalRef<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "({}) {} ({})",
            match self {
                PrincipalRef::Group(..) => "group",
                PrincipalRef::ServicePrincipal(..) => "service principal",
                PrincipalRef::User(..) => "user",
            },
            self.display_name(),
            self.id()
        ))
    }
}
