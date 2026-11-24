use crate::AzureDevOpsGroup;
use facet::Facet;
use std::sync::Arc;

/// A page from [Graph Groups List (7.1)](https://learn.microsoft.com/en-us/rest/api/azure/devops/graph/groups/list?view=azure-devops-rest-7.1#examples).
#[derive(Debug, Facet)]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureDevOpsGroupListResponse {
    /// Count in the documented collection envelope. The legacy `graphGroups`
    /// envelope can omit it.
    pub count: Option<u32>,
    #[facet(alias = "graphGroups")]
    pub value: Vec<Arc<AzureDevOpsGroup>>,
}
