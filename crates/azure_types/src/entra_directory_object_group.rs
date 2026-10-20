use crate::EntraGroupDisplayName;
use crate::EntraGroupId;
use arbitrary::Arbitrary;

/// Identity properties of a Microsoft Graph v1.0 [group](https://learn.microsoft.com/en-us/graph/api/resources/group?view=graph-rest-1.0#properties).
///
/// The name remains unknown for limited-information responses. This shallow
/// projection intentionally ignores other properties in unrestricted responses.
#[derive(Debug, Clone, Eq, PartialEq, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct EntraDirectoryObjectGroup {
    pub display_name: Option<EntraGroupDisplayName>,
    pub id: EntraGroupId,
}

cloud_terrastodon_registry::register_thing!(EntraDirectoryObjectGroup);
cloud_terrastodon_registry::register_arbitrary!(EntraDirectoryObjectGroup);
