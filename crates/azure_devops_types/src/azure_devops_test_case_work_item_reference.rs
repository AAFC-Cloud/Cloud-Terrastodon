use crate::AzureDevOpsWorkItemId;
use arbitrary::Arbitrary;
use cloud_terrastodon_azure_types::OptionalNonEmptyStringProxy;

#[derive(Debug, Eq, PartialEq, Clone, Arbitrary, facet::Facet)]
pub struct AzureDevOpsTestCaseWorkItemReference {
    // The Test API encodes this work item ID as a JSON string.
    // https://learn.microsoft.com/en-us/rest/api/azure/devops/test/test-suites/get?view=azure-devops-rest-7.1#workitemreference
    #[facet(default, proxy = OptionalNonEmptyStringProxy)]
    pub id: Option<AzureDevOpsWorkItemId>,
    pub name: Option<String>,
    #[facet(rename = "type")]
    pub wtype: Option<String>,
    pub url: Option<String>,
    #[facet(rename = "webUrl")]
    pub web_url: Option<String>,
}
