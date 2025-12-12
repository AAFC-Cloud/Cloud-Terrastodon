use crate::user_id::UserId;
use arbitrary::Arbitrary;
use cloud_terrastodon_hcl_types::prelude::AzureAdResourceBlockKind;
use cloud_terrastodon_hcl_types::prelude::HclImportBlock;
use cloud_terrastodon_hcl_types::prelude::HclProviderReference;
use cloud_terrastodon_hcl_types::prelude::ResourceBlockReference;
use cloud_terrastodon_hcl_types::prelude::Sanitizable;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Arbitrary)]
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
impl From<User> for HclImportBlock {
    fn from(user: User) -> Self {
        HclImportBlock {
            provider: HclProviderReference::Inherited,
            id: user.id.to_string(),
            to: ResourceBlockReference::AzureAD {
                kind: AzureAdResourceBlockKind::User,
                name: format!("{}__{}", user.user_principal_name, user.id).sanitize(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use eyre::Result;
    use uuid::Uuid;

    #[test]
    fn deserializes() -> Result<()> {
        let expanded = "55555555-5555-5555-5555-555555555555";
        let id: Uuid = serde_json::from_str(serde_json::to_string(expanded)?.as_str())?;
        assert_eq!(id.to_string(), expanded);

        Ok(())
    }
}
