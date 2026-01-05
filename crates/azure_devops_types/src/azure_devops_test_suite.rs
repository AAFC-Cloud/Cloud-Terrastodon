use super::azure_devops_test_plan::AzureDevOpsTestPlanIdentityRef;
use super::azure_devops_test_plan::AzureDevOpsTestPlanShallowReference;
use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
pub struct AzureDevOpsTestSuite {
    #[serde(rename = "areaUri")]
    pub area_uri: Option<String>,
    pub children: Option<Vec<AzureDevOpsTestSuite>>,
    #[serde(rename = "defaultConfigurations")]
    pub default_configurations: Option<Vec<AzureDevOpsTestPlanShallowReference>>,
    #[serde(rename = "defaultTesters")]
    pub default_testers: Option<Vec<AzureDevOpsTestPlanShallowReference>>,
    pub id: u32,
    #[serde(rename = "inheritDefaultConfigurations")]
    pub inherit_default_configurations: Option<bool>,
    #[serde(rename = "lastError")]
    pub last_error: Option<String>,
    #[serde(rename = "lastPopulatedDate")]
    pub last_populated_date: Option<DateTime<Utc>>,
    #[serde(rename = "lastUpdatedBy")]
    pub last_updated_by: Option<AzureDevOpsTestPlanIdentityRef>,
    #[serde(rename = "lastUpdatedDate")]
    pub last_updated_date: Option<DateTime<Utc>>,
    pub name: String,
    pub parent: Option<AzureDevOpsTestPlanShallowReference>,
    pub plan: Option<AzureDevOpsTestPlanShallowReference>,
    pub project: Option<AzureDevOpsTestPlanShallowReference>,
    #[serde(rename = "queryString")]
    pub query_string: Option<String>,
    #[serde(rename = "requirementId")]
    pub requirement_id: Option<u32>,
    pub revision: Option<u32>,
    pub state: Option<String>,
    #[serde(rename = "suiteType")]
    pub suite_type: Option<String>,
    pub suites: Option<Vec<AzureDevOpsTestPlanShallowReference>>,
    #[serde(rename = "testCaseCount")]
    pub test_case_count: Option<u32>,
    #[serde(rename = "testCasesUrl")]
    pub test_cases_url: Option<String>,
    pub text: Option<String>,
    pub url: Option<String>,
}
