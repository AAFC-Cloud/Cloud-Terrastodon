use crate::EntraDirectoryObjectUrl;
use arbitrary::Arbitrary;

/// Reference body for [adding a group member](https://learn.microsoft.com/graph/api/group-post-members?view=graph-rest-1.0#request-body).
#[derive(Debug, Clone, Arbitrary, facet::Facet)]
pub struct EntraGroupMemberReference {
    #[facet(rename = "@odata.id")]
    pub member: EntraDirectoryObjectUrl,
}

cloud_terrastodon_registry::register_thing!(EntraGroupMemberReference);
cloud_terrastodon_registry::register_arbitrary!(EntraGroupMemberReference);
