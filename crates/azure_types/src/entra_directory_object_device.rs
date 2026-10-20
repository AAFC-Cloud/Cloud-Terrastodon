use crate::EntraDeviceObjectId;
use arbitrary::Arbitrary;

/// Identity properties of a Microsoft Graph v1.0 [device](https://learn.microsoft.com/en-us/graph/api/resources/device?view=graph-rest-1.0#properties).
///
/// The name remains unknown for limited-information responses. This shallow
/// projection intentionally ignores other properties in unrestricted responses.
#[derive(Debug, Clone, Eq, PartialEq, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct EntraDirectoryObjectDevice {
    pub display_name: Option<String>,
    pub id: EntraDeviceObjectId,
}

cloud_terrastodon_registry::register_thing!(EntraDirectoryObjectDevice);
cloud_terrastodon_registry::register_arbitrary!(EntraDirectoryObjectDevice);
