use crate::ArbitraryJson;
use eyre::Result;
use facet::Facet;
use jiff::Timestamp;
use jiff::ZonedDateTime;
use jiff::tz::TimeZone;
use std::collections::HashMap;
use uuid::Uuid;

// https://learn.microsoft.com/en-us/rest/api/cost-management/query/usage?view=rest-cost-management-2024-08-01&tabs=HTTP#querydataset

impl CostManagementQueryDefinition {
    pub fn new_cost_total_this_month() -> Result<Self> {
        Ok(CostManagementQueryDefinition {
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
            time_period: CostManagementQueryTimePeriod::get_month_start_and_today_end()?,
        })
    }
    pub fn new_cost_by_day_this_month() -> Result<Self> {
        Ok(CostManagementQueryDefinition {
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
            time_period: CostManagementQueryTimePeriod::get_month_start_and_today_end()?,
        })
    }
    pub fn new_cost_by_resource_group_this_month() -> Result<Self> {
        Ok(CostManagementQueryDefinition {
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
            time_period: CostManagementQueryTimePeriod::get_month_start_and_today_end()?,
        })
    }
}

impl CostManagementQueryTimePeriod {
    pub fn get_month_start_and_today_end() -> Result<Self> {
        let today = ZonedDateTime::now().date();
        // Select dates in the system time zone and express their calendar
        // boundaries as UTC timestamps in the Cost Management query.
        Ok(CostManagementQueryTimePeriod {
            from: TimeZone::UTC.to_timestamp(today.first_of_month().at(0, 0, 0, 0))?,
            to: TimeZone::UTC.to_timestamp(today.at(23, 59, 59, 0))?,
        })
    }
}

#[derive(Debug, Clone, Facet)]
pub struct CostManagementErrorDetails {
    pub code: String,
    pub message: String,
}
#[derive(Debug, Clone, Facet)]
pub struct CostManagementErrorResponse {
    pub error: CostManagementErrorDetails,
}
#[derive(Debug, Clone, Facet)]
#[repr(C)]
pub enum CostManagementExportType {
    ActualCost,
    AmortizedCost,
    Usage,
}

#[derive(Debug, Clone, Facet)]
#[repr(C)]
pub enum CostManagementFunctionType {
    Sum,
}
#[derive(Debug, Clone, Facet)]
#[repr(C)]
pub enum CostManagementQueryDatasetGranularityType {
    Daily,
    None,
}
#[derive(Debug, Clone, Facet)]
pub struct CostManagementQueryAggregation {
    pub name: String,
    pub function: CostManagementFunctionType,
}

#[derive(Debug, Clone, Facet)]
pub struct CostManagementQueryColumn {
    pub name: String,
    #[facet(rename = "type")]
    pub kind: String,
}
#[derive(Debug, Clone, Facet)]
#[repr(C)]
pub enum CostManagementQueryColumnType {
    Dimension,
    TagKey,
}

#[derive(Debug, Clone, Facet)]
pub struct CostManagementQueryComparisonExpression {
    pub name: String,
    pub operator: CostManagementQueryOperatorType,
    pub values: Vec<String>,
}
#[derive(Debug, Clone, Facet)]
pub struct CostManagementQueryDataset {
    pub granularity: CostManagementQueryDatasetGranularityType,
    pub aggregation: HashMap<String, CostManagementQueryAggregation>,
    pub configuration: Option<QueryDatasetConfiguration>,
    pub filter: Option<CostManagementQueryFilter>,
    pub grouping: Option<Vec<CostManagementQueryGrouping>>,
    pub sorting: Option<Vec<CostManagementQuerySorting>>,
}
#[derive(Debug, Clone, Facet)]
pub struct CostManagementQuerySorting {
    pub direction: CostManagementQuerySortingDirection,
    pub name: String,
}

#[derive(Debug, Clone, Facet)]
#[repr(C)]
pub enum CostManagementQuerySortingDirection {
    Descending,
    Ascending,
}

#[derive(Debug, Clone, Facet)]
pub struct QueryDatasetConfiguration {
    pub columns: Vec<String>,
}

#[derive(Debug, Clone, Facet)]
pub struct CostManagementQueryDefinition {
    pub dataset: CostManagementQueryDataset,
    pub timeframe: CostManagementTimeframeType,
    #[facet(rename = "type")]
    pub kind: CostManagementExportType,
    #[facet(rename = "timePeriod")]
    pub time_period: CostManagementQueryTimePeriod,
}
#[derive(Debug, Clone, Facet)]
pub struct CostManagementQueryFilter {
    pub and: Vec<CostManagementQueryFilter>,
    pub dimensions: CostManagementQueryComparisonExpression,
    pub or: Vec<CostManagementQueryFilter>,
    pub tags: CostManagementQueryComparisonExpression,
}

#[derive(Debug, Clone, Facet)]
pub struct CostManagementQueryGrouping {
    pub name: String,
    #[facet(rename = "type")]
    pub kind: CostManagementQueryColumnType,
}
#[derive(Debug, Clone, Facet)]
#[repr(C)]
pub enum CostManagementQueryOperatorType {
    In,
}

#[derive(Debug, Clone, Facet)]
pub struct CostManagementQueryResult {
    #[facet(rename = "eTag")]
    pub e_tag: Option<String>,
    pub id: String,
    pub location: Option<String>,
    pub name: Uuid,
    pub properties: CostManagementQueryResultProperties,
    pub sku: Option<String>,
    pub tags: Option<ArbitraryJson>,
    #[facet(rename = "type")]
    pub kind: String,
}
#[derive(Debug, Clone, Facet)]
pub struct CostManagementQueryResultProperties {
    pub columns: Vec<CostManagementQueryColumn>,
    #[facet(rename = "nextLink")]
    pub next_link: Option<String>,
    pub rows: Vec<ArbitraryJson>,
}
#[derive(Debug, Clone, Facet)]
pub struct CostManagementQueryTimePeriod {
    pub from: Timestamp,
    pub to: Timestamp,
}

#[derive(Debug, Clone, Facet)]
#[repr(C)]
pub enum CostManagementTimeframeType {
    BillingMonthToDate,
    Custom,
    MonthToDate,
    TheLastBillingMonth,
    TheLastMonth,
    WeekToDate,
}
