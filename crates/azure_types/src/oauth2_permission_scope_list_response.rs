use crate::OAuth2PermissionScope;
use facet::Facet;
use std::sync::Arc;
use url::Url;

/// [Selected service principal OAuth2 permission scopes](https://learn.microsoft.com/graph/api/serviceprincipal-get?view=graph-rest-1.0).
#[derive(Facet)]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct OAuth2PermissionScopeListResponse {
    #[facet(rename = "oauth2PermissionScopes")]
    pub oauth2_permission_scopes: Arc<Vec<OAuth2PermissionScope>>,
    #[facet(rename = "@odata.context")]
    pub context: Option<Url>,
}
