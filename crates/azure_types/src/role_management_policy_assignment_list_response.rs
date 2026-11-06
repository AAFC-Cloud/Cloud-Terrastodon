use crate::RoleManagementPolicyAssignment;
use facet::Facet;
use std::sync::Arc;

/// [Role management policy assignment collection](https://learn.microsoft.com/rest/api/authorization/role-management-policy-assignments/list-for-scope?view=rest-authorization-2020-10-01#rolemanagementpolicyassignmentlistresult).
#[derive(Facet)]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct RoleManagementPolicyAssignmentListResponse {
    pub value: Vec<Arc<RoleManagementPolicyAssignment>>,
    #[facet(rename = "nextLink")]
    pub next_link: Option<String>,
}
