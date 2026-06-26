use crate::user_id::EntraUserId;
use arbitrary::Arbitrary;
use cloud_terrastodon_hcl_types::AzureAdResourceBlockKind;
use cloud_terrastodon_hcl_types::HclImportBlock;
use cloud_terrastodon_hcl_types::HclProviderReference;
use cloud_terrastodon_hcl_types::ResourceBlockReference;
use cloud_terrastodon_hcl_types::Sanitizable;

#[derive(Debug, PartialEq, Eq, Clone, Arbitrary, facet::Facet)]
pub struct EntraUser {
    #[facet(rename = "businessPhones")]
    pub business_phones: Vec<String>,
    #[facet(rename = "displayName")]
    pub display_name: String,
    #[facet(rename = "givenName")]
    pub given_name: Option<String>,
    pub id: EntraUserId,
    #[facet(rename = "jobTitle")]
    pub job_title: Option<String>,
    pub mail: Option<String>,
    #[facet(rename = "otherMails")]
    #[facet(default)]
    pub other_mails: Vec<String>,
    #[facet(rename = "mobilePhone")]
    pub mobile_phone: Option<String>,
    #[facet(rename = "officeLocation")]
    pub office_location: Option<String>,
    #[facet(rename = "preferredLanguage")]
    pub preferred_language: Option<String>,
    pub surname: Option<String>,
    #[facet(rename = "userPrincipalName")]
    pub user_principal_name: String,
}
impl std::fmt::Display for EntraUser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.id.to_string().as_str())?;
        f.write_str(" - ")?;
        f.write_str(&self.user_principal_name)?;
        Ok(())
    }
}
impl From<EntraUser> for HclImportBlock {
    fn from(user: EntraUser) -> Self {
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
        let id: Uuid = facet_json::from_str(&facet_json::to_string(expanded)?)?;
        assert_eq!(id.to_string(), expanded);

        Ok(())
    }
}

cloud_terrastodon_registry::register_thing!(EntraUser);
