use crate::EntraGroup;
use crate::EntraPrincipal;
use crate::EntraPrincipalId;
use crate::EntraPrincipalKind;
use crate::EntraServicePrincipal;
use crate::EntraUser;
use uuid::Uuid;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum EntraPrincipalRef<'a> {
    User(&'a EntraUser),
    Group(&'a EntraGroup),
    ServicePrincipal(&'a EntraServicePrincipal),
    // Device(&'a Value),
}
impl<'a> From<&'a EntraUser> for EntraPrincipalRef<'a> {
    fn from(value: &'a EntraUser) -> Self {
        Self::User(value)
    }
}
impl<'a> From<&'a EntraGroup> for EntraPrincipalRef<'a> {
    fn from(value: &'a EntraGroup) -> Self {
        Self::Group(value)
    }
}
impl<'a> From<&'a EntraServicePrincipal> for EntraPrincipalRef<'a> {
    fn from(value: &'a EntraServicePrincipal) -> Self {
        Self::ServicePrincipal(value)
    }
}
impl<'a> From<&'a EntraPrincipal> for EntraPrincipalRef<'a> {
    fn from(value: &'a EntraPrincipal) -> Self {
        match value {
            EntraPrincipal::User(user) => EntraPrincipalRef::User(user),
            EntraPrincipal::Group(group) => EntraPrincipalRef::Group(group),
            EntraPrincipal::ServicePrincipal(service_principal) => {
                EntraPrincipalRef::ServicePrincipal(service_principal)
            }
        }
    }
}
impl AsRef<Uuid> for EntraPrincipalRef<'_> {
    fn as_ref(&self) -> &Uuid {
        match self {
            EntraPrincipalRef::User(user) => &user.id,
            EntraPrincipalRef::Group(group) => &group.id,
            EntraPrincipalRef::ServicePrincipal(service_principal) => &service_principal.id,
        }
    }
}
impl EntraPrincipalRef<'_> {
    pub fn display_name(&self) -> &str {
        match self {
            EntraPrincipalRef::User(user) => user.user_principal_name.as_str(),
            EntraPrincipalRef::Group(group) => group.display_name.as_str(),
            EntraPrincipalRef::ServicePrincipal(service_principal) => {
                &service_principal.display_name
            }
        }
    }
    pub fn name(&self) -> &str {
        match self {
            EntraPrincipalRef::User(user) => user.user_principal_name.as_str(),
            EntraPrincipalRef::Group(group) => group.display_name.as_str(),
            EntraPrincipalRef::ServicePrincipal(service_principal) => {
                &service_principal.display_name
            }
        }
    }
    pub fn id(&self) -> EntraPrincipalId {
        match self {
            EntraPrincipalRef::User(user) => EntraPrincipalId::UserId(user.id),
            EntraPrincipalRef::Group(group) => EntraPrincipalId::GroupId(group.id),
            EntraPrincipalRef::ServicePrincipal(service_principal) => {
                EntraPrincipalId::ServicePrincipalId(service_principal.id)
            }
        }
    }
    pub fn kind(&self) -> EntraPrincipalKind {
        match self {
            EntraPrincipalRef::User(..) => EntraPrincipalKind::User,
            EntraPrincipalRef::Group(..) => EntraPrincipalKind::Group,
            EntraPrincipalRef::ServicePrincipal(..) => EntraPrincipalKind::ServicePrincipal,
        }
    }
    pub fn as_user(&self) -> Option<&EntraUser> {
        match self {
            EntraPrincipalRef::User(user) => Some(user),
            _ => None,
        }
    }
    pub fn as_group(&self) -> Option<&EntraGroup> {
        match self {
            EntraPrincipalRef::Group(group) => Some(group),
            _ => None,
        }
    }
    pub fn as_service_principal(&self) -> Option<&EntraServicePrincipal> {
        match self {
            EntraPrincipalRef::ServicePrincipal(service_principal) => Some(service_principal),
            _ => None,
        }
    }
}
impl std::fmt::Display for EntraPrincipalRef<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "({}) {} ({})",
            match self {
                EntraPrincipalRef::Group(..) => "group",
                EntraPrincipalRef::ServicePrincipal(..) => "service principal",
                EntraPrincipalRef::User(..) => "user",
            },
            self.display_name(),
            self.id()
        ))
    }
}
