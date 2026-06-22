use super::azure_devops_test_plan::AzureDevOpsTestPlanIdentityRef;
use crate::AzureDevOpsTestPlanShallowReference;

#[derive(Debug, Eq, PartialEq, Clone, facet::Facet)]
pub struct AzureDevOpsTestCasePointAssignment {
    pub configuration: Option<AzureDevOpsTestPlanShallowReference>,
    pub tester: Option<AzureDevOpsTestPlanIdentityRef>,
}

#[derive(Debug, Eq, PartialEq, Clone, facet::Facet)]
pub struct AzureDevOpsTestCaseWorkItemReference {
    pub id: Option<String>,
    pub name: Option<String>,
    #[facet(rename = "type")]
    pub wtype: Option<String>,
    pub url: Option<String>,
    #[facet(rename = "webUrl")]
    pub web_url: Option<String>,
}

#[derive(Debug, Eq, PartialEq, Clone, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct SuiteTestCase {
    pub point_assignments: Option<Vec<AzureDevOpsTestCasePointAssignment>>,
    pub test_case: AzureDevOpsTestCaseWorkItemReference,
}
