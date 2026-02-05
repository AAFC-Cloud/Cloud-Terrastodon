use crate::prelude::EntraGroup;
use crate::prelude::PrincipalId;
use crate::prelude::EntraServicePrincipal;
use crate::prelude::EntraUser;
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, strum::EnumDiscriminants)]
#[strum_discriminants(name(PrincipalKind))]
#[strum_discriminants(derive(strum::Display))]
#[serde(tag = "@odata.type")]
pub enum Principal {
    #[serde(rename = "#microsoft.graph.user")]
    User(EntraUser),
    #[serde(rename = "#microsoft.graph.group")]
    Group(EntraGroup),
    #[serde(rename = "#microsoft.graph.servicePrincipal")]
    ServicePrincipal(Box<EntraServicePrincipal>),
    // #[serde(rename = "#microsoft.graph.device")]
    // Device(Value),
}
impl From<EntraUser> for Principal {
    fn from(value: EntraUser) -> Self {
        Self::User(value)
    }
}
impl From<EntraGroup> for Principal {
    fn from(value: EntraGroup) -> Self {
        Self::Group(value)
    }
}
impl From<EntraServicePrincipal> for Principal {
    fn from(value: EntraServicePrincipal) -> Self {
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
    pub fn name(&self) -> &str {
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
    pub fn kind(&self) -> PrincipalKind {
        PrincipalKind::from(self)
    }
    pub fn as_user(&self) -> Option<&EntraUser> {
        match self {
            Principal::User(user) => Some(user),
            _ => None,
        }
    }
    pub fn as_group(&self) -> Option<&EntraGroup> {
        match self {
            Principal::Group(group) => Some(group),
            _ => None,
        }
    }
    pub fn as_service_principal(&self) -> Option<&EntraServicePrincipal> {
        match self {
            Principal::ServicePrincipal(service_principal) => Some(service_principal),
            _ => None,
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
    use crate::user_id::EntraUserId;
    use serde_json::Value;

    #[test]
    fn it_works() -> eyre::Result<()> {
        let user: EntraUser = EntraUser {
            business_phones: vec![],
            display_name: "User, Fake".to_string(),
            given_name: Some("User".to_string()),
            id: EntraUserId::new(Uuid::nil()),
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
