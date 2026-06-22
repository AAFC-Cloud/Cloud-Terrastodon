use crate::AzureDevOpsDescriptor;
use facet_json::RawJson;

#[derive(Debug, Clone, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureDevOpsGroup {
    pub description: String,
    pub descriptor: AzureDevOpsDescriptor,
    pub display_name: String,
    pub domain: String,
    pub is_cross_project: Option<bool>,
    pub is_deleted: Option<bool>,
    pub is_global_scope: Option<bool>,
    pub is_restricted_visible: Option<bool>,
    pub legacy_descriptor: Option<RawJson<'static>>,
    pub local_scope_id: Option<RawJson<'static>>,
    pub mail_address: Option<String>,
    pub origin: String,
    pub origin_id: String,
    pub principal_name: String,
    pub scope_id: Option<RawJson<'static>>,
    pub scope_name: Option<RawJson<'static>>,
    pub scope_type: Option<RawJson<'static>>,
    pub securing_host_id: Option<RawJson<'static>>,
    pub special_type: Option<RawJson<'static>>,
    pub subject_kind: String,
    pub url: String,
}
