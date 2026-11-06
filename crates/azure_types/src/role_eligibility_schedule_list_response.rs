use crate::RoleEligibilitySchedule;
use std::sync::Arc;

/// [Role eligibility schedule collection](https://learn.microsoft.com/rest/api/authorization/role-eligibility-schedules/list-for-scope?view=rest-authorization-2020-10-01#roleeligibilityschedulelistresult).
#[derive(facet::Facet)]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct RoleEligibilityScheduleListResponse {
    pub value: Vec<Arc<RoleEligibilitySchedule>>,
    #[facet(rename = "nextLink")]
    pub next_link: Option<String>,
}
