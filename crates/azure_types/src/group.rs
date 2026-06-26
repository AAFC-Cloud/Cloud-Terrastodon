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
