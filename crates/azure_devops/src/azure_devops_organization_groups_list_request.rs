use crate::azure_devops_groups_list_request::collect_graph_groups;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureDevOpsAuthSelection;
use cloud_terrastodon_azure_devops_types::AzureDevOpsGroup;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_pathing::sanitize_windows_path_component;
use eyre::Result;
use facet::Facet;
use std::path::PathBuf;
use std::sync::Arc;

/// Lists groups from [Azure DevOps Graph (7.1)](https://learn.microsoft.com/en-us/rest/api/azure/devops/graph/groups/list?view=azure-devops-rest-7.1) at organization scope.
#[derive(Debug, Clone, Arbitrary, Facet)]
#[facet(traits(Clone))]
pub struct AzureDevOpsOrganizationGroupsListRequest {
    pub organization: AzureDevOpsOrganizationUrl,
    pub authentication: AzureDevOpsAuthSelection,
}

impl HasCacheKey for AzureDevOpsOrganizationGroupsListRequest {
    fn cache_key(&self) -> Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az/devops/org",
            &sanitize_windows_path_component(
                &self.organization.organization_name()?.to_ascii_lowercase(),
            ),
            "group/list",
        ]))
    }
}

impl InvalidatesOwnCacheKey for AzureDevOpsOrganizationGroupsListRequest {}

impl ExecutableRequest for AzureDevOpsOrganizationGroupsListRequest {
    type Error = eyre::Report;
    type Output = Vec<Arc<AzureDevOpsGroup>>;

    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> Result<CloudTerrastodonExecutor> {
        self.organization.organization_name()?;
        executor.validate_authentication_target(
            &self.authentication.into(),
            &self.organization.api_url("dev.azure.com", "_apis", &[])?,
        )?;
        executor.with_authentication_lease(&self.authentication.into())
    }

    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            collect_graph_groups(
                &self.organization,
                None,
                self.authentication,
                &self.cache_key()?,
                CacheOperationOwner::of::<Self>(),
                &executor,
            )
            .await
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsOrganizationGroupsListRequest);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsOrganizationGroupsListRequest);
cloud_terrastodon_registry_execution::register_executable_request!(
    AzureDevOpsOrganizationGroupsListRequest => Vec<Arc<AzureDevOpsGroup>>,
    effects = [Read]
);
