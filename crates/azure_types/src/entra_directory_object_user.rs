use crate::EntraUserId;
use crate::UserPrincipalName;
use arbitrary::Arbitrary;

/// Identity properties of a Microsoft Graph v1.0 [user](https://learn.microsoft.com/en-us/graph/api/resources/user?view=graph-rest-1.0#properties).
///
/// Directory-object collections can return only `id` and `@odata.type` when
/// permission to read the resource is missing. Optional properties retain that
/// limited-information state. This is an intentional shallow projection;
/// additional properties in unrestricted Graph responses are ignored.
#[derive(Debug, Clone, Eq, PartialEq, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct EntraDirectoryObjectUser {
    pub display_name: Option<String>,
    pub id: EntraUserId,
    pub mail: Option<String>,
    pub user_principal_name: Option<UserPrincipalName>,
}

cloud_terrastodon_registry::register_thing!(EntraDirectoryObjectUser);
cloud_terrastodon_registry::register_arbitrary!(EntraDirectoryObjectUser);
