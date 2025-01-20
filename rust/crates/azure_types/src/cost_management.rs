use chrono::DateTime;
use chrono::Datelike;
use chrono::Local;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

// https://learn.microsoft.com/en-us/rest/api/cost-management/query/usage?view=rest-cost-management-2024-08-01&tabs=HTTP#querydataset

impl QueryDefinition {
    pub fn new_cost_total_this_month() -> Self {
        QueryDefinition {
            kind: ExportType::ActualCost,
            dataset: QueryDataset {
                granularity: GranularityType::None,
                aggregation: vec![(
                    "totalCost".to_string(),
                    QueryAggregation {
                        name: "Cost".to_string(),
                        function: FunctionType::Sum,
                    },
                )]
                .into_iter()
                .collect(),
                sorting: Some(vec![QuerySorting {
                    direction: Direction::Descending,
                    name: "Cost".to_string(),
                }]),
                configuration: None,
                filter: None,
                grouping: None,
            },
            timeframe: TimeframeType::Custom,
            time_period: QueryTimePeriod::get_month_start_and_today_end(),
        }
    }
    pub fn new_cost_by_day_this_month() -> Self {
        QueryDefinition {
            kind: ExportType::ActualCost,
            dataset: QueryDataset {
                granularity: GranularityType::Daily,
                aggregation: vec![(
                    "totalCost".to_string(),
                    QueryAggregation {
                        name: "Cost".to_string(),
                        function: FunctionType::Sum,
                    },
                )]
                .into_iter()
                .collect(),
                sorting: Some(vec![QuerySorting {
                    direction: Direction::Ascending,
                    name: "UsageDate".to_string(),
                }]),
                configuration: None,
                filter: None,
                grouping: None,
            },
            timeframe: TimeframeType::Custom,
            time_period: QueryTimePeriod::get_month_start_and_today_end(),
        }
    }
    pub fn new_cost_by_resource_group_this_month() -> Self {
        QueryDefinition {
            kind: ExportType::ActualCost,
            dataset: QueryDataset {
                granularity: GranularityType::None,
                aggregation: vec![(
                    "totalCost".to_string(),
                    QueryAggregation {
                        name: "Cost".to_string(),
                        function: FunctionType::Sum,
                    },
                )]
                .into_iter()
                .collect(),
                sorting: Some(vec![QuerySorting {
                    direction: Direction::Descending,
                    name: "Cost".to_string(),
                }]),
                configuration: None,
                filter: None,
                grouping: Some(vec![
                    QueryGrouping {
                        kind: QueryColumnType::Dimension,
                        name: "ResourceGroupName".to_string(),
                    },
                    QueryGrouping {
                        kind: QueryColumnType::Dimension,
                        name: "SubscriptionId".to_string(),
                    },
                ]),
            },
            timeframe: TimeframeType::Custom,
            time_period: QueryTimePeriod::get_month_start_and_today_end(),
        }
    }
}

impl QueryTimePeriod {
    pub fn get_month_start_and_today_end() -> Self {
        let today = Local::now().date_naive();
        let start_of_month = today.with_day(1).unwrap().and_hms_opt(0, 0, 0).unwrap();
        let end_of_today = today.and_hms_opt(23, 59, 59).unwrap();

        QueryTimePeriod {
            from: start_of_month.and_utc(),
            to: end_of_today.and_utc(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetails {
    pub code: String,
    pub message: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: ErrorDetails,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportType {
    ActualCost,
    AmortizedCost,
    Usage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FunctionType {
    Sum,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GranularityType {
    Daily,
    None,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryAggregation {
    pub name: String,
    pub function: FunctionType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryColumn {
    pub name: String,
    #[serde(rename = "type")]
    pub kind: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryColumnType {
    Dimension,
    TagKey,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryComparisonExpression {
    pub name: String,
    pub operator: QueryOperatorType,
    pub values: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryDataset {
    pub granularity: GranularityType,
    pub aggregation: HashMap<String, QueryAggregation>,
    pub configuration: Option<QueryDatasetConfiguration>,
    pub filter: Option<QueryFilter>,
    pub grouping: Option<Vec<QueryGrouping>>,
    pub sorting: Option<Vec<QuerySorting>>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuerySorting {
    pub direction: Direction,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Direction {
    Descending,
    Ascending,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryDatasetConfiguration {
    pub columns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryDefinition {
    pub dataset: QueryDataset,
    pub timeframe: TimeframeType,
    #[serde(rename = "type")]
    pub kind: ExportType,
    #[serde(rename = "timePeriod")]
    pub time_period: QueryTimePeriod,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryFilter {
    pub and: Vec<QueryFilter>,
    pub dimensions: QueryComparisonExpression,
    pub or: Vec<QueryFilter>,
    pub tags: QueryComparisonExpression,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryGrouping {
    pub name: String,
    #[serde(rename = "type")]
    pub kind: QueryColumnType,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryOperatorType {
    In,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    #[serde(rename = "eTag")]
    pub e_tag: Option<String>,
    pub id: String,
    pub location: Option<String>,
    pub name: Uuid,
    pub properties: QueryResultProperties,
    pub sku: Option<String>,
    pub tags: Option<Value>,
    #[serde(rename = "type")]
    pub kind: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResultProperties {
    pub columns: Vec<QueryColumn>,
    #[serde(rename = "nextLink")]
    pub next_link: Option<String>,
    pub rows: Vec<Value>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryTimePeriod {
    pub from: DateTime<Utc>,
    pub to: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimeframeType {
    BillingMonthToDate,
    Custom,
    MonthToDate,
    TheLastBillingMonth,
    TheLastMonth,
    WeekToDate,
}
