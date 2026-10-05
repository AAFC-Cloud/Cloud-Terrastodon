use crate::AzureDevOpsReferenceLinkUrl;
use arbitrary::Arbitrary;

/// A hypermedia reference returned by Azure DevOps.
///
/// Microsoft defines its optional `href` in the official [ReferenceLink source](https://github.com/microsoft/azure-devops-node-api/blob/master/api/interfaces/common/VSSInterfaces.ts).
/// The owning relation name is stored by [`crate::AzureDevOpsReferenceLinks`].
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, facet::Facet)]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureDevOpsReferenceLink {
    pub href: Option<AzureDevOpsReferenceLinkUrl>,
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsReferenceLink);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsReferenceLink);
