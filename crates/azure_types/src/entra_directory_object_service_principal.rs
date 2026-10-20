use crate::EntraServicePrincipalObjectId;
use arbitrary::Arbitrary;

/// Identity properties of a Microsoft Graph v1.0 [service principal](https://learn.microsoft.com/en-us/graph/api/resources/serviceprincipal?view=graph-rest-1.0#properties).
///
/// The name remains unknown for limited-information responses. This shallow
/// projection intentionally ignores other properties in unrestricted responses.
#[derive(Debug, Clone, Eq, PartialEq, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct EntraDirectoryObjectServicePrincipal {
    pub display_name: Option<String>,
    pub id: EntraServicePrincipalObjectId,
}

cloud_terrastodon_registry::register_thing!(EntraDirectoryObjectServicePrincipal);
cloud_terrastodon_registry::register_arbitrary!(EntraDirectoryObjectServicePrincipal);
