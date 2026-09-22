use crate::QueryResultType;
use crate::QueryType;
use crate::WorkItemFieldReference;
use crate::WorkItemLink;
use crate::WorkItemQuerySortColumn;
use crate::WorkItemReference;
use arbitrary::Arbitrary;
use chrono::DateTime;
use chrono::Utc;

/// The result of a work item query.
/// https://learn.microsoft.com/en-us/rest/api/azure/devops/wit/wiql/query-by-id?view=azure-devops-rest-7.1
/// https://learn.microsoft.com/en-us/rest/api/azure/devops/wit/wiql/query-by-wiql?view=azure-devops-rest-7.1&tabs=HTTP#workitemqueryresult
#[derive(Debug, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct WorkItemQueryResult {
    /// The date the query was run in the context of.
    pub as_of: DateTime<Utc>,
    /// The columns of the query.
    pub columns: Vec<WorkItemFieldReference>,
    /// The result type.
    pub query_result_type: QueryResultType,
    /// The type of the query.
    pub query_type: QueryType,
    /// The sort columns of the query.
    pub sort_columns: Option<Vec<WorkItemQuerySortColumn>>,
    /// The work item links returned by the query.
    pub work_item_relations: Option<Vec<WorkItemLink>>,
    /// The work items returned by the query.
    #[facet(default)]
    pub work_items: Vec<WorkItemReference>,
}

cloud_terrastodon_registry::register_thing!(WorkItemQueryResult);
cloud_terrastodon_registry::register_arbitrary!(WorkItemQueryResult);
cloud_terrastodon_registry::register_arbitrary!(Option<WorkItemQueryResult>);
