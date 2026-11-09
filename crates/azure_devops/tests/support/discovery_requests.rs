//! Synthetic data-only contracts for offline registry integration.
//!
//! Project discovery and selection use real SDK requests. This remaining folder
//! contract exercises later data-only inputs without a second service transport.

use cloud_terrastodon_auth_types::AzureDevOpsAuthSelection;
use cloud_terrastodon_azure_devops_types::AzureDevOpsBuildFolder;
use cloud_terrastodon_azure_devops_types::AzureDevOpsBuildFolderPath;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsProjectArgument;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use facet::Facet;
use std::sync::Arc;

#[derive(Clone, Debug, Facet)]
pub struct ProjectFoldersRequest {
    pub org_url: AzureDevOpsOrganizationUrl,
    pub project: AzureDevOpsProjectArgument<'static>,
    pub auth_selection: AzureDevOpsAuthSelection,
    pub path: Option<AzureDevOpsBuildFolderPath>,
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

offline_request!(ProjectFoldersRequest => Arc<Vec<Arc<AzureDevOpsBuildFolder>>>);

cloud_terrastodon_registry_execution::register_executable_request!(ProjectFoldersRequest => Arc<Vec<Arc<AzureDevOpsBuildFolder>>>, effects = [Read]);
