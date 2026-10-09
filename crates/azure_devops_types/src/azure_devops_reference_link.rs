use crate::AzureDevOpsReferenceLinkUrl;
use arbitrary::Arbitrary;

/// A hypermedia reference returned by Azure DevOps.
///
/// Microsoft REST documentation: [ReferenceLinks (7.1)](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/definitions/get?view=azure-devops-rest-7.1#referencelinks)
/// describes the containing collection; its individual link shape is documented
/// by the SDK source below.
/// Microsoft defines its optional `href` in the official [ReferenceLink source](https://github.com/microsoft/azure-devops-node-api/blob/master/api/interfaces/common/VSSInterfaces.ts).
/// The owning relation name is stored by [`crate::AzureDevOpsReferenceLinks`].
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, facet::Facet)]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureDevOpsReferenceLink {
    pub href: Option<AzureDevOpsReferenceLinkUrl>,
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsReferenceLink);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsReferenceLink);
