use crate::EntraUserId;
use crate::UserPrincipalName;
use arbitrary::Arbitrary;
use facet::Facet;

/// The ID, mail, and sign-in name of the [signed-in Graph user](https://learn.microsoft.com/en-us/graph/api/user-get?view=graph-rest-1.0).
#[derive(Arbitrary, Facet)]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct EntraSignedInUser {
    #[facet(rename = "@odata.context")]
    pub context: String,
    pub id: EntraUserId,
    #[facet(sensitive)]
    pub mail: Option<String>,
    #[facet(rename = "userPrincipalName", sensitive)]
    pub user_principal_name: UserPrincipalName,
}

cloud_terrastodon_registry::register_thing!(EntraSignedInUser);
cloud_terrastodon_registry::register_arbitrary!(EntraSignedInUser);
