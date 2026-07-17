use crate::EntraServicePrincipalObjectId;
use crate::EntraUserId;
use arbitrary::Arbitrary;
use itertools::Itertools;

// https://learn.microsoft.com/en-us/graph/api/resources/oauth2permissiongrant?view=graph-rest-1.0

#[derive(Debug, Clone, Eq, PartialEq, Arbitrary, facet::Facet)]
#[facet(transparent)]
pub struct OAuth2PermissionGrantId(pub String);
impl std::fmt::Display for OAuth2PermissionGrantId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl AsRef<str> for OAuth2PermissionGrantId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Eq, PartialEq, Arbitrary, facet::Facet)]
#[repr(C)]
pub enum ConsentType {
    AllPrincipals,
    Principal,
}

#[derive(Debug, Eq, PartialEq, Arbitrary, facet::Facet)]
pub struct OAuth2PermissionGrant {
    /// The object id (not appId) of the client service principal for the application that's authorized to act on behalf of a signed-in user when accessing an API. Required. Supports `$filter` (`eq` only).
    #[facet(rename = "clientId")]
    pub client_id: EntraServicePrincipalObjectId,

    /// Indicates if authorization is granted for the client application to impersonate all users or only a specific user. AllPrincipals indicates authorization to impersonate all users. Principal indicates authorization to impersonate a specific user. Consent on behalf of all users can be granted by an administrator. Nonadmin users might be authorized to consent on behalf of themselves in some cases, for some delegated permissions. Required. Supports `$filter` (`eq` only).
    #[facet(rename = "consentType")]
    pub consent_type: ConsentType,

    /// Unique identifier for the oAuth2PermissionGrant. Read-only.
    pub id: OAuth2PermissionGrantId,

    /// The id of the user on behalf of whom the client is authorized to access the resource, when consentType is Principal. If consentType is AllPrincipals this value is null. Required when consentType is Principal. Supports `$filter` (`eq` only).
    #[facet(rename = "principalId", default)]
    pub principal_id: Option<EntraUserId>,

    /// The id of the resource service principal to which access is authorized. This identifies the API that the client is authorized to attempt to call on behalf of a signed-in user. Supports `$filter` (`eq` only).
    #[facet(rename = "resourceId")]
    pub resource_id: EntraServicePrincipalObjectId,

    /// A space-separated list of the claim values for delegated permissions that should be included in access tokens for the resource application (the API). For example, `openid User.Read GroupMember.Read.All`. Each claim value should match the value field of one of the delegated permissions defined by the API, listed in the oauth2PermissionScopes property of the resource service principal. Must not exceed 3,850 characters in length.
    pub scope: String,
}
impl OAuth2PermissionGrant {
    pub fn get_scopes(&self) -> Vec<&str> {
        self.scope.split_ascii_whitespace().collect_vec()
    }
    // pub fn new(
    //     service_principal_id: ServicePrincipalId,
    //     consent_type: ConsentType,

    // )
}

cloud_terrastodon_registry::register_thing!(OAuth2PermissionGrantId);
cloud_terrastodon_registry::register_arbitrary!(OAuth2PermissionGrantId);

cloud_terrastodon_registry::register_thing!(OAuth2PermissionGrant);
cloud_terrastodon_registry::register_arbitrary!(OAuth2PermissionGrant);

cloud_terrastodon_registry::register_arbitrary!(Vec<OAuth2PermissionGrant>);
