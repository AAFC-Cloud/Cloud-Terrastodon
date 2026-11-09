use crate::ArbitraryJson;
use crate::EntraGroupDisplayName;
use crate::EntraGroupId;
use arbitrary::Arbitrary;
use chrono::DateTime;
use chrono::Utc;
use cloud_terrastodon_hcl_types::AzureAdResourceBlockKind;
use cloud_terrastodon_hcl_types::HclImportBlock;
use cloud_terrastodon_hcl_types::HclProviderReference;
use cloud_terrastodon_hcl_types::ResourceBlockReference;
use cloud_terrastodon_hcl_types::Sanitizable;
use facet::Facet;

/// An Entra [group](https://learn.microsoft.com/en-us/graph/api/resources/group?view=graph-rest-1.0)
/// returned by Microsoft Graph v1.0.
#[derive(Debug, Clone, PartialEq, Eq, Facet)]
#[facet(rename_all = "camelCase")]
pub struct EntraGroup {
    pub classification: Option<ArbitraryJson>,
    pub created_date_time: Option<DateTime<Utc>>,
    pub creation_options: Vec<String>,
    pub deleted_date_time: Option<DateTime<Utc>>,
    pub description: Option<String>,
    pub display_name: EntraGroupDisplayName,
    pub expiration_date_time: Option<DateTime<Utc>>,
    pub group_types: Vec<String>,
    pub id: EntraGroupId,
    pub is_assignable_to_role: Option<bool>,
    pub mail: Option<String>,
    pub mail_enabled: Option<bool>,
    pub mail_nickname: Option<String>,
    pub membership_rule: Option<ArbitraryJson>,
    pub membership_rule_processing_state: Option<ArbitraryJson>,
    pub on_premises_domain_name: Option<String>,
    pub on_premises_last_sync_date_time: Option<DateTime<Utc>>,
    pub on_premises_net_bios_name: Option<String>,
    pub on_premises_provisioning_errors: Vec<ArbitraryJson>,
    pub on_premises_sam_account_name: Option<String>,
    pub on_premises_security_identifier: Option<String>,
    pub on_premises_sync_enabled: Option<bool>,
    pub preferred_data_location: Option<String>,
    pub preferred_language: Option<String>,
    pub proxy_addresses: Vec<String>,
    pub renewed_date_time: Option<DateTime<Utc>>,
    pub resource_behavior_options: Vec<String>,
    pub resource_provisioning_options: Vec<String>,
    pub security_enabled: bool,
    pub security_identifier: String,
    pub service_provisioning_errors: Vec<ArbitraryJson>,
    pub theme: Option<ArbitraryJson>,
    pub unique_name: Option<String>,
    pub visibility: Option<String>,
}
impl<'a> Arbitrary<'a> for EntraGroup {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            classification: arbitrary_optional_json(u)?,
            created_date_time: Option::<DateTime<Utc>>::arbitrary(u)?,
            creation_options: Vec::<String>::arbitrary(u)?,
            deleted_date_time: Option::<DateTime<Utc>>::arbitrary(u)?,
            description: Option::<String>::arbitrary(u)?,
            display_name: EntraGroupDisplayName::arbitrary(u)?,
            expiration_date_time: Option::<DateTime<Utc>>::arbitrary(u)?,
            group_types: Vec::<String>::arbitrary(u)?,
            id: EntraGroupId::arbitrary(u)?,
            is_assignable_to_role: Option::<bool>::arbitrary(u)?,
            mail: Option::<String>::arbitrary(u)?,
            mail_enabled: Option::<bool>::arbitrary(u)?,
            mail_nickname: Option::<String>::arbitrary(u)?,
            membership_rule: arbitrary_optional_json(u)?,
            membership_rule_processing_state: arbitrary_optional_json(u)?,
            on_premises_domain_name: Option::<String>::arbitrary(u)?,
            on_premises_last_sync_date_time: Option::<DateTime<Utc>>::arbitrary(u)?,
            on_premises_net_bios_name: Option::<String>::arbitrary(u)?,
            on_premises_provisioning_errors: arbitrary_json_vec(u)?,
            on_premises_sam_account_name: Option::<String>::arbitrary(u)?,
            on_premises_security_identifier: Option::<String>::arbitrary(u)?,
            on_premises_sync_enabled: Option::<bool>::arbitrary(u)?,
            preferred_data_location: Option::<String>::arbitrary(u)?,
            preferred_language: Option::<String>::arbitrary(u)?,
            proxy_addresses: Vec::<String>::arbitrary(u)?,
            renewed_date_time: Option::<DateTime<Utc>>::arbitrary(u)?,
            resource_behavior_options: Vec::<String>::arbitrary(u)?,
            resource_provisioning_options: Vec::<String>::arbitrary(u)?,
            security_enabled: bool::arbitrary(u)?,
            security_identifier: String::arbitrary(u)?,
            service_provisioning_errors: arbitrary_json_vec(u)?,
            theme: arbitrary_optional_json(u)?,
            unique_name: Option::<String>::arbitrary(u)?,
            visibility: Option::<String>::arbitrary(u)?,
        })
    }
}

fn arbitrary_optional_json<'a>(
    u: &mut arbitrary::Unstructured<'a>,
) -> arbitrary::Result<Option<ArbitraryJson>> {
    Ok(Option::<String>::arbitrary(u)?
        .as_deref()
        .map(ArbitraryJson::from_text))
}

fn arbitrary_json_vec<'a>(
    u: &mut arbitrary::Unstructured<'a>,
) -> arbitrary::Result<Vec<ArbitraryJson>> {
    Ok(Vec::<String>::arbitrary(u)?
        .into_iter()
        .map(|value| ArbitraryJson::from_text(&value))
        .collect())
}
impl std::fmt::Display for EntraGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.display_name)?;
        f.write_str(" (")?;
        f.write_str(self.id.to_string().as_str())?;
        f.write_str(")")?;
        Ok(())
    }
}
impl From<EntraGroup> for HclImportBlock {
    fn from(group: EntraGroup) -> Self {
        Self::from(&group)
    }
}
impl From<&EntraGroup> for HclImportBlock {
    fn from(group: &EntraGroup) -> Self {
        HclImportBlock {
            provider: HclProviderReference::Inherited,
            id: format!("/groups/{}", group.id.0.as_hyphenated()),
            to: ResourceBlockReference::AzureAD {
                kind: AzureAdResourceBlockKind::Group,
                name: format!("{}__{}", group.display_name, group.id).sanitize(),
            },
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
        let id: EntraGroupId = facet_json::from_str(&facet_json::to_string(expanded)?)?;
        assert_eq!(id.to_string(), expanded);

        Ok(())
    }
}

cloud_terrastodon_registry::register_thing!(EntraGroup);
cloud_terrastodon_registry::register_arbitrary!(EntraGroup);
