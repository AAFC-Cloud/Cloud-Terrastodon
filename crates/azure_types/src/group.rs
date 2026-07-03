use arbitrary::Arbitrary;
use crate::EntraGroupId;
use chrono::DateTime;
use chrono::Utc;
use cloud_terrastodon_hcl_types::AzureAdResourceBlockKind;
use cloud_terrastodon_hcl_types::HclImportBlock;
use cloud_terrastodon_hcl_types::HclProviderReference;
use cloud_terrastodon_hcl_types::ResourceBlockReference;
use cloud_terrastodon_hcl_types::Sanitizable;
use facet_json::RawJson;

#[derive(Debug, Clone, PartialEq, Eq, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct EntraGroup {
    pub classification: Option<RawJson<'static>>,
    pub created_date_time: Option<DateTime<Utc>>,
    pub creation_options: Vec<String>,
    pub deleted_date_time: Option<DateTime<Utc>>,
    pub description: Option<String>,
    pub display_name: String,
    pub expiration_date_time: Option<DateTime<Utc>>,
    pub group_types: Vec<String>,
    pub id: EntraGroupId,
    pub is_assignable_to_role: Option<bool>,
    pub mail: Option<String>,
    pub mail_enabled: Option<bool>,
    pub mail_nickname: Option<String>,
    pub membership_rule: Option<RawJson<'static>>,
    pub membership_rule_processing_state: Option<RawJson<'static>>,
    pub on_premises_domain_name: Option<String>,
    pub on_premises_last_sync_date_time: Option<DateTime<Utc>>,
    pub on_premises_net_bios_name: Option<String>,
    pub on_premises_provisioning_errors: Vec<RawJson<'static>>,
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
    pub service_provisioning_errors: Vec<RawJson<'static>>,
    pub theme: Option<RawJson<'static>>,
    pub unique_name: Option<String>,
    pub visibility: Option<String>,
}
impl<'a> Arbitrary<'a> for EntraGroup {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            classification: arbitrary_optional_raw_json(u)?,
            created_date_time: Option::<DateTime<Utc>>::arbitrary(u)?,
            creation_options: Vec::<String>::arbitrary(u)?,
            deleted_date_time: Option::<DateTime<Utc>>::arbitrary(u)?,
            description: Option::<String>::arbitrary(u)?,
            display_name: String::arbitrary(u)?,
            expiration_date_time: Option::<DateTime<Utc>>::arbitrary(u)?,
            group_types: Vec::<String>::arbitrary(u)?,
            id: EntraGroupId::arbitrary(u)?,
            is_assignable_to_role: Option::<bool>::arbitrary(u)?,
            mail: Option::<String>::arbitrary(u)?,
            mail_enabled: Option::<bool>::arbitrary(u)?,
            mail_nickname: Option::<String>::arbitrary(u)?,
            membership_rule: arbitrary_optional_raw_json(u)?,
            membership_rule_processing_state: arbitrary_optional_raw_json(u)?,
            on_premises_domain_name: Option::<String>::arbitrary(u)?,
            on_premises_last_sync_date_time: Option::<DateTime<Utc>>::arbitrary(u)?,
            on_premises_net_bios_name: Option::<String>::arbitrary(u)?,
            on_premises_provisioning_errors: arbitrary_raw_json_vec(u)?,
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
            service_provisioning_errors: arbitrary_raw_json_vec(u)?,
            theme: arbitrary_optional_raw_json(u)?,
            unique_name: Option::<String>::arbitrary(u)?,
            visibility: Option::<String>::arbitrary(u)?,
        })
    }
}

fn arbitrary_optional_raw_json<'a>(
    u: &mut arbitrary::Unstructured<'a>,
) -> arbitrary::Result<Option<RawJson<'static>>> {
    Option::<String>::arbitrary(u)?
        .map(|value| {
            facet_json::to_string(&value)
                .map(RawJson::from_owned)
                .map_err(|_| arbitrary::Error::IncorrectFormat)
        })
        .transpose()
}

fn arbitrary_raw_json_vec<'a>(
    u: &mut arbitrary::Unstructured<'a>,
) -> arbitrary::Result<Vec<RawJson<'static>>> {
    Vec::<String>::arbitrary(u)?
        .into_iter()
        .map(|value| {
            facet_json::to_string(&value)
                .map(RawJson::from_owned)
                .map_err(|_| arbitrary::Error::IncorrectFormat)
        })
        .collect()
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

