use super::azure_devops_test_plan::AzureDevOpsTestPlanIdentityRef;
use crate::prelude::AzureDevOpsTestPlanShallowReference;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
pub struct AzureDevOpsTestCasePointAssignment {
    pub configuration: Option<AzureDevOpsTestPlanShallowReference>,
    pub tester: Option<AzureDevOpsTestPlanIdentityRef>,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
pub struct AzureDevOpsTestCaseWorkItemReference {
    pub id: Option<String>,
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub wtype: Option<String>,
    pub url: Option<String>,
    #[serde(rename = "webUrl")]
    pub web_url: Option<String>,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
pub struct SuiteTestCase {
    #[serde(rename = "pointAssignments")]
    pub point_assignments: Option<Vec<AzureDevOpsTestCasePointAssignment>>,
    #[serde(rename = "testCase")]
    pub test_case: AzureDevOpsTestCaseWorkItemReference,
}
