use super::azure_devops_test_plan::AzureDevOpsTestPlanIdentityRef;
use crate::AzureDevOpsTestCaseWorkItemReference;
use crate::AzureDevOpsTestPlanShallowReference;
use arbitrary::Arbitrary;

#[derive(Debug, Eq, PartialEq, Clone, Arbitrary, facet::Facet)]
pub struct AzureDevOpsTestCasePointAssignment {
    pub configuration: Option<AzureDevOpsTestPlanShallowReference>,
    pub tester: Option<AzureDevOpsTestPlanIdentityRef>,
}

#[derive(Debug, Eq, PartialEq, Clone, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct SuiteTestCase {
    pub point_assignments: Option<Vec<AzureDevOpsTestCasePointAssignment>>,
    pub test_case: AzureDevOpsTestCaseWorkItemReference,
}

cloud_terrastodon_registry::register_thing!(SuiteTestCase);
cloud_terrastodon_registry::register_arbitrary!(SuiteTestCase);
cloud_terrastodon_registry::register_arbitrary!(Vec<SuiteTestCase>);
