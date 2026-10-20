use crate::EntraOrganizationalContactId;
use arbitrary::Arbitrary;

/// Identity properties of a Microsoft Graph v1.0 [organizational contact](https://learn.microsoft.com/en-us/graph/api/resources/orgcontact?view=graph-rest-1.0#properties).
///
/// Organizational contacts differ from personal Outlook contacts and do not
/// represent security principals. Missing names and mail remain unknown for
/// limited-information responses. This shallow projection intentionally ignores
/// other properties in unrestricted responses.
#[derive(Debug, Clone, Eq, PartialEq, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct EntraDirectoryObjectOrganizationalContact {
    pub display_name: Option<String>,
    pub id: EntraOrganizationalContactId,
    pub mail: Option<String>,
}

cloud_terrastodon_registry::register_thing!(EntraDirectoryObjectOrganizationalContact);
cloud_terrastodon_registry::register_arbitrary!(EntraDirectoryObjectOrganizationalContact);
