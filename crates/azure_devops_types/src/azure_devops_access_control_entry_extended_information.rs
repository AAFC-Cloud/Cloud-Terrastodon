use facet::Facet;

/// Extended access-control metadata from [Access Control Lists Query (7.1)](https://learn.microsoft.com/en-us/rest/api/azure/devops/security/access-control-lists/query?view=azure-devops-rest-7.1#aceextendedinformation).
#[derive(Debug, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureDevOpsAccessControlEntryExtendedInformation {
    pub effective_allow: Option<i32>,
    pub effective_deny: Option<i32>,
    pub inherited_allow: Option<i32>,
    pub inherited_deny: Option<i32>,
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsAccessControlEntryExtendedInformation);
