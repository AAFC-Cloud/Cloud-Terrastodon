use cloud_terrastodon_azure_devops_types::AzureDevOpsDescriptor;
use cloud_terrastodon_azure_devops_types::AzureDevOpsIdentityPermissionReport;
use facet::Facet;

/// A matching [Azure DevOps Graph identity](https://learn.microsoft.com/en-us/rest/api/azure/devops/graph/users/list?view=azure-devops-rest-7.1)
/// and the outcome of reading its permissions.
#[derive(Debug, Facet)]
pub struct AzureDevOpsEntraPermissionIdentity<Failure> {
    pub display_name: String,
    pub descriptor: AzureDevOpsDescriptor,
    pub result: Result<AzureDevOpsIdentityPermissionReport, Failure>,
}
