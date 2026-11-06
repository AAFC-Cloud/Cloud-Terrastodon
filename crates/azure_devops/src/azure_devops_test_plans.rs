use arbitrary::Arbitrary;
use cloud_terrastodon_auth::AzureDevOpsAuthContext;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops_types::AzureDevOpsTestPlan;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_execution::RequestExecution;
use cloud_terrastodon_rest::MicrosoftContinuationToken;
use cloud_terrastodon_rest::RestRequest;
use facet::Facet;
use reqwest::Method;
use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::debug;

#[derive(Debug, Clone, facet::Facet)]
#[facet(traits(Clone))]
pub struct AzureDevOpsTestPlanListRequest<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    pub project: AzureDevOpsProjectArgument<'a>,
    pub auth_context: Cow<'a, AzureDevOpsAuthContext>,
}

#[track_caller]
pub fn fetch_azure_devops_test_plans<'a>(
    org_url: &'a AzureDevOpsOrganizationUrl,
    project: impl Into<AzureDevOpsProjectArgument<'a>>,
    auth_context: &'a AzureDevOpsAuthContext,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, AzureDevOpsTestPlanListRequest<'a>> {
    executor.execute(AzureDevOpsTestPlanListRequest {
        org_url: Cow::Borrowed(org_url),
        project: project.into(),
        auth_context: Cow::Borrowed(auth_context),
    })
}

impl<'a> Arbitrary<'a> for AzureDevOpsTestPlanListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            project: AzureDevOpsProjectArgument::arbitrary(u)?.into_owned(),
            auth_context: Cow::Owned(AzureDevOpsAuthContext::None),
        })
    }
}
impl<'a> HasCacheKey for AzureDevOpsTestPlanListRequest<'a> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "devops",
            self.org_url.name(),
            "test",
            "plan",
            "list",
            self.project.to_string().as_ref(),
        ]))
    }
}

impl<'a> InvalidatesOwnCacheKey for AzureDevOpsTestPlanListRequest<'a> {}

impl<'a> ExecutableRequest for AzureDevOpsTestPlanListRequest<'a> {
    type Error = eyre::Report;
    type Output = Vec<Arc<AzureDevOpsTestPlan>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            debug!("Fetching Azure DevOps test plans");
            #[derive(Facet)]
            #[facet(rename_all = "camelCase")]
            struct Response {
                count: u32,
                value: Vec<Arc<AzureDevOpsTestPlan>>,
            }

            let mut plans = Vec::new();
            let mut continuation: Option<MicrosoftContinuationToken> = None;
            let mut count = 0;
            let cache_key = self.cache_key()?;
            let mut page_index = 0;
            loop {
                let project = self.project.to_string();
                let mut query = vec![("api-version", "7.1-preview.1")];
                if let Some(token) = continuation.as_ref() {
                    query.push(("continuationToken", token.as_str()));
                }
                let url = self.org_url.api_url(
                    "dev.azure.com",
                    &format!("{}/_apis/testplan/plans", project),
                    &query,
                )?;
                let request = RestRequest::from_method_and_url(Method::GET, url)?
                    .cache(cache_key.join(page_index.to_string())?)
                    .azure_devops_auth_context(self.auth_context.as_ref())?;
                let (response, next_continuation) = request
                    .receive_with_ms_continuation_token_cached::<Response>(
                        &executor,
                        CacheOperationOwner::of::<AzureDevOpsTestPlanListRequest<'static>>(),
                    )
                    .await?;
                count += response.count;
                plans.extend(response.value.iter().cloned());
                continuation = next_continuation;
                page_index += 1;
                if continuation.is_none() {
                    break;
                }
            }
            debug!("Found {count} Azure DevOps test plans");
            Ok(plans)
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsTestPlanListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsTestPlanListRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(AzureDevOpsTestPlanListRequest<'static> => Vec<Arc<AzureDevOpsTestPlan>>, effects = [Read]);

#[cfg(test)]
mod test {
    use super::*;
    use crate::fetch_all_azure_devops_projects;
    use crate::get_default_organization_url;
    use cloud_terrastodon_auth::AuthContext;
    use cloud_terrastodon_command::CancellationToken;
    use eyre::bail;

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
                for project in projects.iter() {
                    let test_plans = fetch_azure_devops_test_plans(
                        &org_url,
                        project.as_ref(),
                        &azure_devops_auth_context,
                        &executor,
                    )
                    .await?;
                    if test_plans.is_empty() {
                        continue;
                    }
                    assert!(
                        test_plans
                            .iter()
                            .all(|test_plan| !test_plan.name.is_empty() && test_plan.id > 0),
                        "Expected sampled Azure DevOps test plans to include names and ids"
                    );
                    return Ok(());
                }

                bail!("Failed to find any test plans in any project");
            })
            .await
    }
}
