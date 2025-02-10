use crate::prelude::ServicePrincipalId;
use crate::prelude::UserId;
use itertools::Itertools;
use serde::Deserialize;
use serde::Serialize;

// https://learn.microsoft.com/en-us/graph/api/resources/oauth2permissiongrant?view=graph-rest-1.0

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct OAuth2PermissionGrantId(String);

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum ConsentType {
    AllPrincipals,
    Principal,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct OAuth2PermissionGrant {
    /// The object id (not appId) of the client service principal for the application that's authorized to act on behalf of a signed-in user when accessing an API. Required. Supports `$filter` (`eq` only).
    #[serde(rename = "clientId")]
    pub client_id: ServicePrincipalId,

    /// Indicates if authorization is granted for the client application to impersonate all users or only a specific user. AllPrincipals indicates authorization to impersonate all users. Principal indicates authorization to impersonate a specific user. Consent on behalf of all users can be granted by an administrator. Nonadmin users might be authorized to consent on behalf of themselves in some cases, for some delegated permissions. Required. Supports `$filter` (`eq` only).
    #[serde(rename = "consentType")]
    pub consent_type: ConsentType,

    /// Unique identifier for the oAuth2PermissionGrant. Read-only.
    pub id: OAuth2PermissionGrantId,

    /// The id of the user on behalf of whom the client is authorized to access the resource, when consentType is Principal. If consentType is AllPrincipals this value is null. Required when consentType is Principal. Supports `$filter` (`eq` only).
    #[serde(rename = "principalId")]
    pub principal_id: Option<UserId>,

    /// The id of the resource service principal to which access is authorized. This identifies the API that the client is authorized to attempt to call on behalf of a signed-in user. Supports `$filter` (`eq` only).
    #[serde(rename = "resourceId")]
    pub resource_id: ServicePrincipalId,

    /// A space-separated list of the claim values for delegated permissions that should be included in access tokens for the resource application (the API). For example, `openid User.Read GroupMember.Read.All`. Each claim value should match the value field of one of the delegated permissions defined by the API, listed in the oauth2PermissionScopes property of the resource service principal. Must not exceed 3,850 characters in length.
    pub scope: String,
}
impl OAuth2PermissionGrant {
    pub fn get_scopes(&self) -> Vec<&str> {
        self.scope.split_ascii_whitespace().collect_vec()
    }
}
