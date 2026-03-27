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

impl CostManagementQueryDefinition {
    pub fn new_cost_total_this_month() -> Self {
        CostManagementQueryDefinition {
            kind: CostManagementExportType::ActualCost,
            dataset: CostManagementQueryDataset {
                granularity: CostManagementQueryDatasetGranularityType::None,
                aggregation: vec![(
                    "totalCost".to_string(),
                    CostManagementQueryAggregation {
                        name: "Cost".to_string(),
                        function: CostManagementFunctionType::Sum,
                    },
                )]
                .into_iter()
                .collect(),
                sorting: Some(vec![CostManagementQuerySorting {
                    direction: CostManagementQuerySortingDirection::Descending,
                    name: "Cost".to_string(),
                }]),
                configuration: None,
                filter: None,
                grouping: None,
            },
            timeframe: CostManagementTimeframeType::Custom,
            time_period: CostManagementQueryTimePeriod::get_month_start_and_today_end(),
        }
    }
    pub fn new_cost_by_day_this_month() -> Self {
        CostManagementQueryDefinition {
            kind: CostManagementExportType::ActualCost,
            dataset: CostManagementQueryDataset {
                granularity: CostManagementQueryDatasetGranularityType::Daily,
                aggregation: vec![(
                    "totalCost".to_string(),
                    CostManagementQueryAggregation {
                        name: "Cost".to_string(),
                        function: CostManagementFunctionType::Sum,
                    },
                )]
                .into_iter()
                .collect(),
                sorting: Some(vec![CostManagementQuerySorting {
                    direction: CostManagementQuerySortingDirection::Ascending,
                    name: "UsageDate".to_string(),
                }]),
                configuration: None,
                filter: None,
                grouping: None,
            },
            timeframe: CostManagementTimeframeType::Custom,
            time_period: CostManagementQueryTimePeriod::get_month_start_and_today_end(),
        }
    }
    pub fn new_cost_by_resource_group_this_month() -> Self {
        CostManagementQueryDefinition {
            kind: CostManagementExportType::ActualCost,
            dataset: CostManagementQueryDataset {
                granularity: CostManagementQueryDatasetGranularityType::None,
                aggregation: vec![(
                    "totalCost".to_string(),
                    CostManagementQueryAggregation {
                        name: "Cost".to_string(),
                        function: CostManagementFunctionType::Sum,
                    },
                )]
                .into_iter()
                .collect(),
                sorting: Some(vec![CostManagementQuerySorting {
                    direction: CostManagementQuerySortingDirection::Descending,
                    name: "Cost".to_string(),
                }]),
                configuration: None,
                filter: None,
                grouping: Some(vec![
                    CostManagementQueryGrouping {
                        kind: CostManagementQueryColumnType::Dimension,
                        name: "ResourceGroupName".to_string(),
                    },
                    CostManagementQueryGrouping {
                        kind: CostManagementQueryColumnType::Dimension,
                        name: "SubscriptionId".to_string(),
                    },
                ]),
            },
            timeframe: CostManagementTimeframeType::Custom,
            time_period: CostManagementQueryTimePeriod::get_month_start_and_today_end(),
        }
    }
}

impl CostManagementQueryTimePeriod {
    pub fn get_month_start_and_today_end() -> Self {
        let today = Local::now().date_naive();
        let start_of_month = today.with_day(1).unwrap().and_hms_opt(0, 0, 0).unwrap();
        let end_of_today = today.and_hms_opt(23, 59, 59).unwrap();

        CostManagementQueryTimePeriod {
            from: start_of_month.and_utc(),
            to: end_of_today.and_utc(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostManagementErrorDetails {
    pub code: String,
    pub message: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostManagementErrorResponse {
    pub error: CostManagementErrorDetails,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CostManagementExportType {
    ActualCost,
    AmortizedCost,
    Usage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CostManagementFunctionType {
    Sum,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CostManagementQueryDatasetGranularityType {
    Daily,
    None,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostManagementQueryAggregation {
    pub name: String,
    pub function: CostManagementFunctionType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostManagementQueryColumn {
    pub name: String,
    #[serde(rename = "type")]
    pub kind: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CostManagementQueryColumnType {
    Dimension,
    TagKey,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostManagementQueryComparisonExpression {
    pub name: String,
    pub operator: CostManagementQueryOperatorType,
    pub values: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostManagementQueryDataset {
    pub granularity: CostManagementQueryDatasetGranularityType,
    pub aggregation: HashMap<String, CostManagementQueryAggregation>,
    pub configuration: Option<QueryDatasetConfiguration>,
    pub filter: Option<CostManagementQueryFilter>,
    pub grouping: Option<Vec<CostManagementQueryGrouping>>,
    pub sorting: Option<Vec<CostManagementQuerySorting>>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostManagementQuerySorting {
    pub direction: CostManagementQuerySortingDirection,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CostManagementQuerySortingDirection {
    Descending,
    Ascending,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryDatasetConfiguration {
    pub columns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostManagementQueryDefinition {
    pub dataset: CostManagementQueryDataset,
    pub timeframe: CostManagementTimeframeType,
    #[serde(rename = "type")]
    pub kind: CostManagementExportType,
    #[serde(rename = "timePeriod")]
    pub time_period: CostManagementQueryTimePeriod,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostManagementQueryFilter {
    pub and: Vec<CostManagementQueryFilter>,
    pub dimensions: CostManagementQueryComparisonExpression,
    pub or: Vec<CostManagementQueryFilter>,
    pub tags: CostManagementQueryComparisonExpression,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostManagementQueryGrouping {
    pub name: String,
    #[serde(rename = "type")]
    pub kind: CostManagementQueryColumnType,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CostManagementQueryOperatorType {
    In,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostManagementQueryResult {
    #[serde(rename = "eTag")]
    pub e_tag: Option<String>,
    pub id: String,
    pub location: Option<String>,
    pub name: Uuid,
    pub properties: CostManagementQueryResultProperties,
    pub sku: Option<String>,
    pub tags: Option<Value>,
    #[serde(rename = "type")]
    pub kind: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostManagementQueryResultProperties {
    pub columns: Vec<CostManagementQueryColumn>,
    #[serde(rename = "nextLink")]
    pub next_link: Option<String>,
    pub rows: Vec<Value>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostManagementQueryTimePeriod {
    pub from: DateTime<Utc>,
    pub to: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CostManagementTimeframeType {
    BillingMonthToDate,
    Custom,
    MonthToDate,
    TheLastBillingMonth,
    TheLastMonth,
    WeekToDate,
}
