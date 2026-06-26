use crate::EntraGroup;
use crate::EntraServicePrincipal;
use crate::EntraUser;
use crate::PrincipalId;
use facet_json::RawJson;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Eq, PartialEq, strum::EnumDiscriminants, facet::Facet)]
#[strum_discriminants(name(PrincipalKind))]
#[strum_discriminants(derive(strum::Display))]
#[facet(opaque, proxy = RawJson<'static>)]
#[repr(C)]
pub enum Principal {
    User(Box<EntraUser>),
    Group(Box<EntraGroup>),
    ServicePrincipal(Box<EntraServicePrincipal>),
}

impl TryFrom<RawJson<'static>> for Principal {
    type Error = eyre::Error;

    fn try_from(value: RawJson<'static>) -> Result<Self, Self::Error> {
        let object = facet_json::from_str::<HashMap<String, RawJson<'static>>>(value.as_str())?;
        let odata_type = object
            .get("@odata.type")
            .ok_or_else(|| eyre::eyre!("Principal missing @odata.type"))?;
        let odata_type = facet_json::from_str::<String>(odata_type.as_str())?;
        Ok(match odata_type.as_str() {
            "#microsoft.graph.user" => Self::User(Box::new(facet_json::from_str(value.as_str())?)),
            "#microsoft.graph.group" => {
                Self::Group(Box::new(facet_json::from_str(value.as_str())?))
            }
            "#microsoft.graph.servicePrincipal" => {
                Self::ServicePrincipal(Box::new(facet_json::from_str(value.as_str())?))
            }
            other => eyre::bail!("Unsupported principal @odata.type: {other}"),
        })
    }
}

impl TryFrom<&Principal> for RawJson<'static> {
    type Error = eyre::Error;

    fn try_from(value: &Principal) -> Result<Self, Self::Error> {
        let (odata_type, inner_json) = match value {
            Principal::User(user) => ("#microsoft.graph.user", facet_json::to_string(user)?),
            Principal::Group(group) => ("#microsoft.graph.group", facet_json::to_string(group)?),
            Principal::ServicePrincipal(service_principal) => (
                "#microsoft.graph.servicePrincipal",
                facet_json::to_string(service_principal)?,
            ),
        };
        let mut object = facet_json::from_str::<HashMap<String, RawJson<'static>>>(&inner_json)?;
        object.insert(
            "@odata.type".to_string(),
            RawJson::from_owned(facet_json::to_string(odata_type)?),
        );
        Ok(RawJson::from_owned(facet_json::to_string(&object)?))
    }
}

impl From<EntraUser> for Principal {
    fn from(value: EntraUser) -> Self {
        Self::User(Box::new(value))
    }
}
impl From<EntraGroup> for Principal {
    fn from(value: EntraGroup) -> Self {
        Self::Group(Box::new(value))
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

    #[test]
    fn it_works() -> eyre::Result<()> {
        let user: EntraUser = EntraUser {
            business_phones: vec![],
            display_name: "User, Fake".to_string(),
            given_name: Some("User".to_string()),
            id: EntraUserId::new(Uuid::nil()),
            job_title: None,
            mail: None,
            other_mails: vec![],
            mobile_phone: None,
            office_location: None,
            preferred_language: None,
            surname: Some("Fake".to_string()),
            user_principal_name: "fake.user@example.com".to_string(),
        };
        let principal = Principal::from(user);
        let encoded = facet_json::to_string_pretty(&principal)?;
        assert!(encoded.contains(r##""@odata.type""##));
        assert!(encoded.contains(r##"#microsoft.graph.user"##));
        let decoded_principal = facet_json::from_str::<Principal>(&encoded)?;
        assert_eq!(principal, decoded_principal);
        Ok(())
    }
}

cloud_terrastodon_registry::register_thing!(Principal);
