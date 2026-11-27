use crate::AzureDevOpsAccessControlEntry;
use crate::AzureDevOpsLegacyIdentityDescriptor;
use facet::Facet;
use std::collections::BTreeMap;

/// The token and explicit entries returned by [Access Control Lists Query (7.1)](https://learn.microsoft.com/en-us/rest/api/azure/devops/security/access-control-lists/query?view=azure-devops-rest-7.1#accesscontrollist).
#[derive(Debug, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureDevOpsAccessControlList {
    pub token: String,
    pub inherit_permissions: bool,
    pub aces_dictionary:
        BTreeMap<AzureDevOpsLegacyIdentityDescriptor, AzureDevOpsAccessControlEntry>,
    pub include_extended_info: Option<bool>,
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsAccessControlList);
