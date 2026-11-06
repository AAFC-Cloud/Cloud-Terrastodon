//! Synthetic data-only contracts for offline registry integration.
//!
//! The SDK's transitional auth-context schemas remain covered by its metadata
//! tests. These requests exercise data-only executor inputs and
//! shared domain collections without implementing a second service transport.

use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureDevOpsAuthSelection;
use cloud_terrastodon_azure_devops_types::AzureDevOpsBuildFolder;
use cloud_terrastodon_azure_devops_types::AzureDevOpsBuildFolderPath;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsProject;
use cloud_terrastodon_azure_devops_types::AzureDevOpsProjectArgument;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use facet::Facet;
use std::sync::Arc;

#[derive(Arbitrary, Clone, Debug, Facet)]
pub struct ProjectInventoryRequest {
    pub org_url: AzureDevOpsOrganizationUrl,
    pub auth_selection: AzureDevOpsAuthSelection,
}

#[derive(Clone, Debug, Facet)]
pub struct ProjectFoldersRequest {
    pub org_url: AzureDevOpsOrganizationUrl,
    pub project: AzureDevOpsProjectArgument<'static>,
    pub auth_selection: AzureDevOpsAuthSelection,
    pub path: Option<AzureDevOpsBuildFolderPath>,
}

#[derive(Clone, Debug, Facet)]
pub struct ProjectSelectionRequest {
    pub org_url: AzureDevOpsOrganizationUrl,
    pub auth_selection: AzureDevOpsAuthSelection,
}

macro_rules! offline_request {
    ($request:ty => $output:ty) => {
        impl ExecutableRequest for $request {
            type Error = eyre::Report;
            type Output = $output;

            fn run<'a>(self, _: CloudTerrastodonExecutor) -> ExecutionFuture<'a, Self::Output>
            where
                Self: 'a,
                Self::Output: 'a,
            {
                panic!("offline discovery must not invoke a request or its authentication provider")
            }
        }
    };
}

offline_request!(ProjectInventoryRequest => Arc<Vec<Arc<AzureDevOpsProject>>>);
offline_request!(ProjectFoldersRequest => Arc<Vec<Arc<AzureDevOpsBuildFolder>>>);
offline_request!(ProjectSelectionRequest => Arc<AzureDevOpsProject>);

cloud_terrastodon_registry_execution::register_executable_request!(ProjectInventoryRequest => Arc<Vec<Arc<AzureDevOpsProject>>>, effects = [Read]);
cloud_terrastodon_registry_execution::register_executable_request!(ProjectFoldersRequest => Arc<Vec<Arc<AzureDevOpsBuildFolder>>>, effects = [Read]);
cloud_terrastodon_registry_execution::register_executable_request!(ProjectSelectionRequest => Arc<AzureDevOpsProject>, effects = [Read, Solicit]);
cloud_terrastodon_registry::register_arbitrary!(ProjectInventoryRequest);
cloud_terrastodon_registry::register_arbitrary!(Arc<Vec<Arc<AzureDevOpsProject>>>);
cloud_terrastodon_registry::register_arbitrary!(Arc<Vec<Arc<AzureDevOpsBuildFolder>>>);
