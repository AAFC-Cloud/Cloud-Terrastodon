use crate::AzureDevOpsDescriptor;
use crate::AzureDevOpsIdentityDisplayName;
use crate::AzureDevOpsIdentityId;
use crate::AzureDevOpsLegacyIdentityDescriptor;
use cloud_terrastodon_azure_types::ArbitraryJson;
use facet::Facet;

/// The legacy identity representation returned by [Read Identities (7.1)](https://learn.microsoft.com/en-us/rest/api/azure/devops/ims/identities/read-identities?view=azure-devops-rest-7.1#identity).
#[derive(Debug, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureDevOpsSecurityIdentity {
    pub custom_display_name: Option<AzureDevOpsIdentityDisplayName>,
    pub descriptor: AzureDevOpsLegacyIdentityDescriptor,
    pub id: AzureDevOpsIdentityId,
    pub is_active: bool,
    pub is_container: Option<bool>,
    pub master_id: Option<AzureDevOpsIdentityId>,
    pub member_ids: Option<Vec<AzureDevOpsIdentityId>>,
    pub member_of: Option<Vec<AzureDevOpsLegacyIdentityDescriptor>>,
    pub members: Option<Vec<AzureDevOpsLegacyIdentityDescriptor>>,
    pub meta_type_id: Option<i32>,
    pub properties: Option<ArbitraryJson>,
    pub provider_display_name: AzureDevOpsIdentityDisplayName,
    pub resource_version: Option<i32>,
    pub social_descriptor: Option<String>,
    pub subject_descriptor: AzureDevOpsDescriptor,
    pub unique_user_id: Option<i32>,
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsSecurityIdentity);
