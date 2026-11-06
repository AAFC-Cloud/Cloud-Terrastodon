use arbitrary::Arbitrary;
use cloud_terrastodon_auth::AzureDevOpsAuthContext;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops_types::SuiteTestCase;
use cloud_terrastodon_azure_types::ArbitraryJson;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_execution::RequestExecution;
use cloud_terrastodon_rest::RestRequest;
use reqwest::Method;
use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::debug;

#[derive(Debug, Clone, facet::Facet)]
#[facet(traits(Clone))]
pub struct AzureDevOpsTestSuiteCasesListRequest<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    pub project: AzureDevOpsProjectArgument<'a>,
    pub plan: String,
    pub suite: String,
    pub auth_context: Cow<'a, AzureDevOpsAuthContext>,
}

#[track_caller]
pub fn fetch_azure_devops_test_suite_cases<'a>(
    org_url: &'a AzureDevOpsOrganizationUrl,
    project: impl Into<AzureDevOpsProjectArgument<'a>>,
    plan: impl Into<String>,
    suite: impl Into<String>,
    auth_context: &'a AzureDevOpsAuthContext,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, AzureDevOpsTestSuiteCasesListRequest<'a>> {
    executor.execute(AzureDevOpsTestSuiteCasesListRequest {
        org_url: Cow::Borrowed(org_url),
        project: project.into(),
        plan: plan.into(),
        suite: suite.into(),
        auth_context: Cow::Borrowed(auth_context),
    })
}

impl<'a> Arbitrary<'a> for AzureDevOpsTestSuiteCasesListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            project: AzureDevOpsProjectArgument::arbitrary(u)?.into_owned(),
            plan: String::arbitrary(u)?,
            suite: String::arbitrary(u)?,
            auth_context: Cow::Owned(AzureDevOpsAuthContext::None),
        })
    }
}
impl<'a> HasCacheKey for AzureDevOpsTestSuiteCasesListRequest<'a> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "devops",
            self.org_url.name(),
            "test",
            "cases",
            "list",
            self.project.to_string().as_ref(),
            self.plan.as_str(),
            self.suite.as_str(),
        ]))
    }
}

impl<'a> InvalidatesOwnCacheKey for AzureDevOpsTestSuiteCasesListRequest<'a> {}

impl<'a> ExecutableRequest for AzureDevOpsTestSuiteCasesListRequest<'a> {
    type Error = eyre::Report;
    type Output = Arc<Vec<SuiteTestCase>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            debug!("Fetching Azure DevOps test cases for suite");

            #[derive(facet::Facet)]
            #[allow(dead_code)]
            struct InvokeResponse {
                continuation_token: Option<ArbitraryJson>,
                count: u32,
                value: Arc<Vec<SuiteTestCase>>,
            }
            let url = format!(
                "{org_url}/{project}/_apis/test/Plans/{planId}/suites/{suiteId}/testcases?api-version=5.0",
                org_url = self.org_url,
                project = self.project,
                planId = self.plan,
                suiteId = self.suite,
            );
            let request = RestRequest::new(Method::GET, url.as_str())?.cache(self.cache_key()?);
            let response = request
                .azure_devops_auth_context(self.auth_context.as_ref())?
                .receive_cached::<InvokeResponse>(
                    &executor,
                    CacheOperationOwner::of::<AzureDevOpsTestSuiteCasesListRequest<'static>>(),
                )
                .await?;
            Ok(Arc::clone(&response.value))
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsTestSuiteCasesListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsTestSuiteCasesListRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(AzureDevOpsTestSuiteCasesListRequest<'static> => Arc<Vec<SuiteTestCase>>, effects = [Read]);

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
                    &azure_devops_auth_context,
                    &executor,
                )
                .await?;
                for project in projects {
                    let plans = crate::fetch_azure_devops_test_plans(
                        &org_url,
                        project.as_ref(),
                        &azure_devops_auth_context,
                        &executor,
                    )
                    .await?;
                    if plans.is_empty() {
                        continue;
                    }
                    for plan in plans.iter().take(3) {
                        let suites = crate::fetch_azure_devops_test_suites(
                            &org_url,
                            project.as_ref(),
                            plan.id.to_string(),
                            &azure_devops_auth_context,
                            &executor,
                        )
                        .await?;
                        for suite in suites.iter().take(3) {
                            let _cases = fetch_azure_devops_test_suite_cases(
                                &org_url,
                                project.as_ref(),
                                plan.id.to_string(),
                                suite.id.to_string(),
                                &azure_devops_auth_context,
                                &executor,
                            )
                            .await?;
                        }
                    }
                    return Ok(());
                }

                bail!("Failed to find any test plans in any project");
            })
            .await
    }
}
