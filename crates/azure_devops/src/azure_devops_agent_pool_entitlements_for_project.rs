use crate::AzureDevOpsProjectArgument;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth::AzureDevOpsAuthContext;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_execution::RequestExecution;
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
    pub auth_context: Cow<'a, AzureDevOpsAuthContext>,
}

#[track_caller]
pub fn fetch_azure_devops_agent_pool_entitlements_for_project<'a>(
    org_url: &'a AzureDevOpsOrganizationUrl,
    project: impl Into<AzureDevOpsProjectArgument<'a>>,
    auth_context: &'a AzureDevOpsAuthContext,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, AzureDevOpsAgentPoolEntitlementListForProjectRequest<'a>> {
    executor.execute(AzureDevOpsAgentPoolEntitlementListForProjectRequest {
        org_url: Cow::Borrowed(org_url),
        project: project.into(),
        auth_context: Cow::Borrowed(auth_context),
    })
}

impl<'a> Arbitrary<'a> for AzureDevOpsAgentPoolEntitlementListForProjectRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            project: AzureDevOpsProjectArgument::arbitrary(u)?.into_owned(),
            auth_context: Cow::Owned(AzureDevOpsAuthContext::None),
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
                .azure_devops_auth_context(self.auth_context.as_ref())?
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::fetch_all_azure_devops_projects;
    use crate::get_default_organization_url;
    use cloud_terrastodon_auth::AuthContext;
    use cloud_terrastodon_command::CancellationToken;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        CancellationToken::new()
            .with_async_scope(|cancellation| async move {
                let auth_context = AuthContext::explicit_azure_cli();
                let azure_devops_auth_context = AzureDevOpsAuthContext::for_tenant(
                    &auth_context,
                    crate::test_tenant_id::test_tenant_id()?,
                )?;
                let executor =
                    crate::test_executor::test_executor(&azure_devops_auth_context, &cancellation)?;
                let org_url = get_default_organization_url(&cancellation).await?;
                let projects = fetch_all_azure_devops_projects(
                    &org_url,
                    azure_devops_auth_context.selection()?,
                    &executor,
                )
                .await?;

                // Iterate projects, and stop when we find the first project with entitlements.
                let mut found = false;
                for project in projects.iter() {
                    let entitlements = fetch_azure_devops_agent_pool_entitlements_for_project(
                        &org_url,
                        &project.name,
                        &azure_devops_auth_context,
                        &executor,
                    )
                    .await?;
                    if !entitlements.is_empty() {
                        assert!(
                            entitlements
                                .iter()
                                .all(|entitlement| !entitlement.name.is_empty()
                                    && !entitlement.pool.name.as_ref().is_empty()),
                            "Expected Azure DevOps queue/pool entitlements to include names"
                        );
                        found = true;
                        break;
                    }
                }

                assert!(
                    found,
                    "Expected at least one Azure DevOps queue/pool entitlement across projects"
                );
                Ok(())
            })
            .await
    }
}
