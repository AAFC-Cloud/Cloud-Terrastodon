use serde::Deserialize;
use serde::Serialize;
use tofu_types::prelude::Sanitizable;
use tofu_types::prelude::TofuAzureADResourceKind;
use tofu_types::prelude::TofuImportBlock;
use tofu_types::prelude::TofuProviderReference;
use tofu_types::prelude::TofuResourceReference;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct User {
    #[serde(rename = "businessPhones")]
    pub business_phones: Vec<String>,
    #[serde(rename = "displayName")]
    pub display_name: String,
    #[serde(rename = "givenName")]
    pub given_name: Option<String>,
    pub id: Uuid,
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
        f.write_str(&self.display_name)?;
        f.write_str(" (")?;
        f.write_str(self.id.to_string().as_str())?;
        f.write_str(")")?;
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
#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn deserializes() -> Result<()> {
        let expanded = "55555555-5555-5555-5555-555555555555";
        let id: Uuid = serde_json::from_str(serde_json::to_string(expanded)?.as_str())?;
        assert_eq!(id.to_string(), expanded);

        Ok(())
    }
}
