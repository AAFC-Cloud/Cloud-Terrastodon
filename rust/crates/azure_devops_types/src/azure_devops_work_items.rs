use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// The result type of a query.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum QueryResultType {
    WorkItem,
    WorkItemLink,
}

/// The type of query.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum QueryType {
    Flat,
    OneHop,
    Tree,
}

/// Reference to a field in a work item.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkItemFieldReference {
    /// The reference name of the field.
    pub reference_name: String,
    /// The friendly name of the field.
    pub name: String,
    /// The REST URL of the resource.
    pub url: String,
}

/// Contains reference to a work item.
#[derive(Debug, Serialize, Deserialize)]
pub struct WorkItemReference {
    /// Work item ID.
    pub id: i32,
    /// REST API URL of the resource.
    pub url: String,
}

/// A sort column.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkItemQuerySortColumn {
    /// The direction to sort by.
    pub descending: bool,
    /// A work item field.
    pub field: WorkItemFieldReference,
}

/// A link between two work items.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkItemLink {
    /// The type of link (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rel: Option<String>,
    /// The source work item (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<WorkItemReference>,
    /// The target work item.
    pub target: WorkItemReference,
}

/// The result of a work item query.
/// https://learn.microsoft.com/en-us/rest/api/azure/devops/wit/wiql/query-by-id?view=azure-devops-rest-7.1
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
    pub sort_columns: Vec<WorkItemQuerySortColumn>,
    /// The work item links returned by the query.
    pub work_item_relations: Vec<WorkItemLink>,
    /// The work items returned by the query.
    pub work_items: Vec<WorkItemReference>,
}
