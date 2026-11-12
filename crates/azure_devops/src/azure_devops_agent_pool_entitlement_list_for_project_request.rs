use crate::AzureDevOpsProjectArgument;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureDevOpsAuthSelection;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_rest::RestRequest;
use facet::Facet;
use reqwest::Method;
use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::debug;

#[derive(Debug, Clone, Facet)]
#[facet(traits(Clone))]
pub struct AzureDevOpsAgentPoolEntitlementListForProjectRequest<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    pub project: AzureDevOpsProjectArgument<'a>,
    pub auth_selection: AzureDevOpsAuthSelection,
}

impl<'a> Arbitrary<'a> for AzureDevOpsAgentPoolEntitlementListForProjectRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            project: AzureDevOpsProjectArgument::arbitrary(u)?.into_owned(),
            auth_selection: AzureDevOpsAuthSelection::arbitrary(u)?,
        })
    }
}
impl<'a> HasCacheKey for AzureDevOpsAgentPoolEntitlementListForProjectRequest<'a> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "devops",
            self.org_url.name(),
            "distributedtask",
            "queue",
            "list",
            "--project",
            &self.project.to_string(),
        ]))
    }
}

impl<'a> InvalidatesOwnCacheKey for AzureDevOpsAgentPoolEntitlementListForProjectRequest<'a> {}

impl<'a> ExecutableRequest for AzureDevOpsAgentPoolEntitlementListForProjectRequest<'a> {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> Result<CloudTerrastodonExecutor, Self::Error> {
        executor.validate_authentication_target(
            &self.auth_selection.into(),
            &self.org_url.api_url("dev.azure.com", "_apis", &[])?,
        )?;
        executor.with_authentication_lease(&self.auth_selection.into())
    }

    type Error = eyre::Report;
    type Output = Vec<Arc<crate::AzureDevOpsAgentPoolEntitlement>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let project = &self.project;
            debug!("Fetching Azure DevOps agent queues (pools) for project {project}");

            #[derive(Facet)]
            struct Response {
                count: u32,
                value: Vec<Arc<crate::AzureDevOpsAgentPoolEntitlement>>,
            }

            let project = project.to_string();
            let url = self.org_url.api_url(
                "dev.azure.com",
                &format!("{project}/_apis/distributedtask/queues"),
                &[("api-version", "7.1")],
            )?;
            let request =
                RestRequest::from_method_and_url(Method::GET, url)?.cache(self.cache_key()?);
            let response = request
                .authentication(self.auth_selection.into())
                .receive_cached::<Response>(
                    &executor,
                    CacheOperationOwner::of::<
                        AzureDevOpsAgentPoolEntitlementListForProjectRequest<'static>,
                    >(),
                )
                .await?;

            debug!(
                "Found {} Azure DevOps agent queue entitlements for project {}",
                response.count, project
            );

            Ok(response.value.to_vec())
        })
    }
}

cloud_terrastodon_registry::register_thing!(
    AzureDevOpsAgentPoolEntitlementListForProjectRequest<'static>
);
cloud_terrastodon_registry::register_arbitrary!(
    AzureDevOpsAgentPoolEntitlementListForProjectRequest<'static>
);
cloud_terrastodon_registry_execution::register_executable_request!(AzureDevOpsAgentPoolEntitlementListForProjectRequest<'static> => Vec<Arc<crate::AzureDevOpsAgentPoolEntitlement>>, effects = [Read]);

cloud_terrastodon_registry_execution::register_runtime_test!(
    AzureDevOpsAgentPoolEntitlementListForProjectRequest<'static>,
    name = "azure-devops-agent-pool-entitlement-list-for-project",
    cached
);
