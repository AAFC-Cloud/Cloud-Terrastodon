use crate::impl_uuid_traits;
use crate::prelude::Fake;
use crate::prelude::UuidWrapper;
use cloud_terrastodon_core_tofu_types::prelude::Sanitizable;
use cloud_terrastodon_core_tofu_types::prelude::TofuAzureADResourceKind;
use cloud_terrastodon_core_tofu_types::prelude::TofuImportBlock;
use cloud_terrastodon_core_tofu_types::prelude::TofuProviderReference;
use cloud_terrastodon_core_tofu_types::prelude::TofuResourceReference;
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct UserId(Uuid);
impl UuidWrapper for UserId {
    fn new(uuid: Uuid) -> Self {
        Self(uuid)
    }

    fn as_ref(&self) -> &Uuid {
        &self.0
    }
}
impl_uuid_traits!(UserId);

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct User {
    #[serde(rename = "businessPhones")]
    pub business_phones: Vec<String>,
    #[serde(rename = "displayName")]
    pub display_name: String,
    #[serde(rename = "givenName")]
    pub given_name: Option<String>,
    pub id: UserId,
    #[serde(rename = "jobTitle")]
    pub job_title: Option<String>,
    pub mail: Option<String>,
    #[serde(rename = "mobilePhone")]
    pub mobile_phone: Option<String>,
    #[serde(rename = "officeLocation")]
    pub office_location: Option<String>,
    #[serde(rename = "preferredLanguage")]
    pub preferred_language: Option<String>,
    pub surname: Option<String>,
    #[serde(rename = "userPrincipalName")]
    pub user_principal_name: String,
}
impl std::fmt::Display for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.id.to_string().as_str())?;
        f.write_str(" - ")?;
        f.write_str(&self.user_principal_name)?;
        Ok(())
    }
}
impl From<User> for TofuImportBlock {
    fn from(user: User) -> Self {
        TofuImportBlock {
            provider: TofuProviderReference::Inherited,
            id: user.id.to_string(),
            to: TofuResourceReference::AzureAD {
                kind: TofuAzureADResourceKind::User,
                name: format!("{}__{}", user.user_principal_name, user.id).sanitize(),
            },
        }
    }
}

impl Fake for User {
    fn fake() -> Self {
        User {
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
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use eyre::Result;

    #[test]
    fn deserializes() -> Result<()> {
        let expanded = "55555555-5555-5555-5555-555555555555";
        let id: Uuid = serde_json::from_str(serde_json::to_string(expanded)?.as_str())?;
        assert_eq!(id.to_string(), expanded);

        Ok(())
    }
}
