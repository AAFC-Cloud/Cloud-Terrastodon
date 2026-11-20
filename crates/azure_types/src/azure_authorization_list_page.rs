use facet::Facet;
use std::sync::Arc;
use url::Url;

/// Paged collection envelope in the Azure Authorization 2022-04-01
/// [role assignment](https://learn.microsoft.com/en-us/rest/api/authorization/role-assignments/list-for-scope?view=rest-authorization-2022-04-01#roleassignmentlistresult),
/// [role definition](https://learn.microsoft.com/en-us/rest/api/authorization/role-definitions/list?view=rest-authorization-2022-04-01#roledefinitionlistresult),
/// and [deny assignment](https://learn.microsoft.com/en-us/rest/api/authorization/deny-assignments/list-for-scope?view=rest-authorization-2022-04-01#denyassignmentlistresult) endpoints.
#[derive(Facet)]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureAuthorizationListPage<T> {
    pub value: Vec<Arc<T>>,
    #[facet(rename = "nextLink")]
    pub next_link: Option<Url>,
}
