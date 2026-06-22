use super::azure_devops_test_plan::AzureDevOpsTestPlanIdentityRef;
use super::azure_devops_test_plan::AzureDevOpsTestPlanShallowReference;
use chrono::DateTime;
use chrono::Utc;

#[derive(Debug, Eq, PartialEq, Clone, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureDevOpsTestSuite {
    pub area_uri: Option<String>,
    #[facet(recursive_type)]
    pub children: Option<Vec<AzureDevOpsTestSuite>>,
    pub default_configurations: Option<Vec<AzureDevOpsTestPlanShallowReference>>,
    pub default_testers: Option<Vec<AzureDevOpsTestPlanShallowReference>>,
    pub id: u32,
    pub inherit_default_configurations: Option<bool>,
    pub last_error: Option<String>,
    pub last_populated_date: Option<DateTime<Utc>>,
    pub last_updated_by: Option<AzureDevOpsTestPlanIdentityRef>,
    pub last_updated_date: Option<DateTime<Utc>>,
    pub name: String,
    pub parent: Option<AzureDevOpsTestPlanShallowReference>,
    pub plan: Option<AzureDevOpsTestPlanShallowReference>,
    pub project: Option<AzureDevOpsTestPlanShallowReference>,
    pub query_string: Option<String>,
    pub requirement_id: Option<u32>,
    pub revision: Option<u32>,
    pub state: Option<String>,
    pub suite_type: Option<String>,
    pub suites: Option<Vec<AzureDevOpsTestPlanShallowReference>>,
    pub test_case_count: Option<u32>,
    pub test_cases_url: Option<String>,
    pub text: Option<String>,
    pub url: Option<String>,
}
