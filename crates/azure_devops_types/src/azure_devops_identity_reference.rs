use crate::AzureDevOpsDescriptor;
use crate::AzureDevOpsIdentityDisplayName;
use crate::AzureDevOpsIdentityId;
use arbitrary::Arbitrary;

/// A shallow reference to an Azure DevOps identity, including people, groups,
/// and processes. Missing identity metadata remains unknown.
///
/// This models the identifying fields of Microsoft's [IdentityRef 7.1 schema](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/folders/list?view=azure-devops-rest-7.1#identityref).
/// Debug decoding rejects fields outside this model, including unmodeled URL,
/// link, and provider metadata.
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureDevOpsIdentityReference {
    pub id: Option<AzureDevOpsIdentityId>,
    pub descriptor: Option<AzureDevOpsDescriptor>,
    pub display_name: Option<AzureDevOpsIdentityDisplayName>,
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsIdentityReference);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsIdentityReference);
