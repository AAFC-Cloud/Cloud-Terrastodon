use crate::EntraGroup;
use crate::EntraPrincipalId;
use crate::EntraServicePrincipal;
use crate::EntraUser;
use arbitrary::Arbitrary;
use facet_json::RawJson;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Eq, PartialEq, strum::EnumDiscriminants, facet::Facet)]
#[strum_discriminants(name(EntraPrincipalKind))]
#[strum_discriminants(derive(strum::Display))]
#[facet(proxy = RawJson<'static>)]
#[repr(C)]
pub enum EntraPrincipal {
    User(Box<EntraUser>),
    Group(Box<EntraGroup>),
    ServicePrincipal(Box<EntraServicePrincipal>),
}

impl<'a> Arbitrary<'a> for EntraPrincipal {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(match u.int_in_range(0..=2)? {
            0 => Self::User(Box::new(EntraUser::arbitrary(u)?)),
            1 => Self::Group(Box::new(EntraGroup::arbitrary(u)?)),
            _ => Self::ServicePrincipal(Box::new(EntraServicePrincipal::arbitrary(u)?)),
        })
    }
}
impl TryFrom<RawJson<'static>> for EntraPrincipal {
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
            other => {
                eyre::bail!("Unsupported principal @odata.type: {other}");
            }
        })
    }
}

impl TryFrom<&EntraPrincipal> for RawJson<'static> {
    type Error = eyre::Error;

    fn try_from(value: &EntraPrincipal) -> Result<Self, Self::Error> {
        let (odata_type, inner_json) = match value {
            EntraPrincipal::User(user) => ("#microsoft.graph.user", facet_json::to_string(user)?),
            EntraPrincipal::Group(group) => {
                ("#microsoft.graph.group", facet_json::to_string(group)?)
            }
            EntraPrincipal::ServicePrincipal(service_principal) => (
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

impl From<EntraUser> for EntraPrincipal {
    fn from(value: EntraUser) -> Self {
        Self::User(Box::new(value))
    }
}
impl From<EntraGroup> for EntraPrincipal {
    fn from(value: EntraGroup) -> Self {
        Self::Group(Box::new(value))
    }
}
impl From<EntraServicePrincipal> for EntraPrincipal {
    fn from(value: EntraServicePrincipal) -> Self {
        Self::ServicePrincipal(Box::new(value))
    }
}
impl TryFrom<EntraPrincipal> for EntraUser {
    type Error = eyre::Error;

    fn try_from(value: EntraPrincipal) -> Result<Self, Self::Error> {
        match value {
            EntraPrincipal::User(user) => Ok(*user),
            other => {
                eyre::bail!("Expected user principal, got {}", other.kind());
            }
        }
    }
}
impl TryFrom<EntraPrincipal> for EntraGroup {
    type Error = eyre::Error;

    fn try_from(value: EntraPrincipal) -> Result<Self, Self::Error> {
        match value {
            EntraPrincipal::Group(group) => Ok(*group),
            other => {
                eyre::bail!("Expected group principal, got {}", other.kind());
            }
        }
    }
}
impl TryFrom<EntraPrincipal> for EntraServicePrincipal {
    type Error = eyre::Error;

    fn try_from(value: EntraPrincipal) -> Result<Self, Self::Error> {
        match value {
            EntraPrincipal::ServicePrincipal(service_principal) => Ok(*service_principal),
            other => {
                eyre::bail!("Expected service principal, got {}", other.kind());
            }
        }
    }
}
impl AsRef<Uuid> for EntraPrincipal {
    fn as_ref(&self) -> &Uuid {
        match self {
            EntraPrincipal::User(user) => &user.id,
            EntraPrincipal::Group(group) => &group.id,
            EntraPrincipal::ServicePrincipal(service_principal) => &service_principal.id,
        }
    }
}
impl EntraPrincipal {
    pub fn display_name(&self) -> &str {
        match self {
            EntraPrincipal::User(user) => user.user_principal_name.as_str(),
            EntraPrincipal::Group(group) => group.display_name.as_str(),
            EntraPrincipal::ServicePrincipal(service_principal) => &service_principal.display_name,
        }
    }
    pub fn name(&self) -> &str {
        match self {
            EntraPrincipal::User(user) => user.user_principal_name.as_str(),
            EntraPrincipal::Group(group) => group.display_name.as_str(),
            EntraPrincipal::ServicePrincipal(service_principal) => &service_principal.display_name,
        }
    }
    pub fn id(&self) -> EntraPrincipalId {
        match self {
            EntraPrincipal::User(user) => EntraPrincipalId::UserId(user.id),
            EntraPrincipal::Group(group) => EntraPrincipalId::GroupId(group.id),
            EntraPrincipal::ServicePrincipal(service_principal) => {
                EntraPrincipalId::ServicePrincipalId(service_principal.id)
            }
        }
    }
    pub fn kind(&self) -> EntraPrincipalKind {
        EntraPrincipalKind::from(self)
    }
    pub fn as_user(&self) -> Option<&EntraUser> {
        match self {
            EntraPrincipal::User(user) => Some(user),
            _ => None,
        }
    }
    pub fn as_group(&self) -> Option<&EntraGroup> {
        match self {
            EntraPrincipal::Group(group) => Some(group),
            _ => None,
        }
    }
    pub fn as_service_principal(&self) -> Option<&EntraServicePrincipal> {
        match self {
            EntraPrincipal::ServicePrincipal(service_principal) => Some(service_principal),
            _ => None,
        }
    }
}
impl std::fmt::Display for EntraPrincipal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "({}) {} ({})",
            match self {
                EntraPrincipal::Group(..) => "group",
                EntraPrincipal::ServicePrincipal(..) => "service principal",
                EntraPrincipal::User(..) => "user",
            },
            self.display_name(),
            self.id()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EntraUserExternalUserState;
    use crate::user_id::EntraUserId;

    #[test]
    fn it_works() -> eyre::Result<()> {
        let user: EntraUser = EntraUser {
            business_phones: vec![],
            display_name: "User, Fake".to_string(),
            external_user_state: Some(EntraUserExternalUserState::Accepted),
            external_user_state_change_date_time: Some(
                chrono::DateTime::from_timestamp(0, 0).expect("Unix epoch is a valid timestamp"),
            ),
            given_name: Some("User".to_string()),
            id: EntraUserId::new(Uuid::nil()),
            job_title: None,
            mail: None,
            other_mails: vec![],
            mobile_phone: None,
            office_location: None,
            preferred_language: None,
            surname: Some("Fake".to_string()),
            user_principal_name: "fake.user@example.com".parse().unwrap(),
        };
        let principal = EntraPrincipal::from(user);
        let encoded = facet_json::to_string_pretty(&principal)?;
        assert!(encoded.contains(r##""@odata.type""##));
        assert!(encoded.contains(r##"#microsoft.graph.user"##));
        let decoded_principal = facet_json::from_str::<EntraPrincipal>(&encoded)?;
        assert_eq!(principal, decoded_principal);
        Ok(())
    }
}

cloud_terrastodon_registry::register_thing!(EntraPrincipal);
cloud_terrastodon_registry::register_arbitrary!(EntraPrincipal);
cloud_terrastodon_registry::register_arbitrary!(Vec<EntraPrincipal>);

cloud_terrastodon_registry::register_from!(EntraUser => EntraPrincipal);
cloud_terrastodon_registry::register_from!(EntraGroup => EntraPrincipal);
cloud_terrastodon_registry::register_from!(EntraServicePrincipal => EntraPrincipal);
cloud_terrastodon_registry::register_try_from!(EntraPrincipal => EntraUser);
cloud_terrastodon_registry::register_try_from!(EntraPrincipal => EntraGroup);
cloud_terrastodon_registry::register_try_from!(EntraPrincipal => EntraServicePrincipal);
