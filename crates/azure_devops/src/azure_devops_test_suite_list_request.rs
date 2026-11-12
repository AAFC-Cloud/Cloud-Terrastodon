use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureDevOpsAuthSelection;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops_types::AzureDevOpsTestSuite;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_rest::MicrosoftContinuationToken;
use cloud_terrastodon_rest::RestRequest;
use facet::Facet;
use reqwest::Method;
use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::debug;

#[derive(Debug, Clone, Facet)]
#[facet(traits(Clone))]
pub struct AzureDevOpsTestSuiteListRequest<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    pub project: AzureDevOpsProjectArgument<'a>,
    pub plan: String,
    pub auth_selection: AzureDevOpsAuthSelection,
}

impl<'a> Arbitrary<'a> for AzureDevOpsTestSuiteListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            project: AzureDevOpsProjectArgument::arbitrary(u)?.into_owned(),
            plan: String::arbitrary(u)?,
            auth_selection: AzureDevOpsAuthSelection::arbitrary(u)?,
        })
    }
}
impl<'a> HasCacheKey for AzureDevOpsTestSuiteListRequest<'a> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "devops",
            self.org_url.name(),
            "test",
            "suite",
            "list",
            self.project.to_string().as_ref(),
            self.plan.as_str(),
        ]))
    }
}

impl<'a> InvalidatesOwnCacheKey for AzureDevOpsTestSuiteListRequest<'a> {}

impl<'a> ExecutableRequest for AzureDevOpsTestSuiteListRequest<'a> {
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
    type Output = Vec<Arc<AzureDevOpsTestSuite>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            debug!("Fetching Azure DevOps test suites");
            #[derive(Facet)]
            #[facet(rename_all = "camelCase")]
            struct Response {
                count: u32,
                value: Vec<Arc<AzureDevOpsTestSuite>>,
            }

            let mut suites = Vec::new();
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
                    &format!("{}/_apis/testplan/Plans/{}/suites", project, self.plan),
                    &query,
                )?;
                let request = RestRequest::from_method_and_url(Method::GET, url)?
                    .cache(cache_key.join(page_index.to_string())?)
                    .authentication(self.auth_selection.into());
                let (response, next_continuation) = request
                    .receive_with_ms_continuation_token_cached::<Response>(
                        &executor,
                        CacheOperationOwner::of::<AzureDevOpsTestSuiteListRequest<'static>>(),
                    )
                    .await?;
                count += response.count;
                suites.extend(response.value.iter().cloned());
                continuation = next_continuation;
                page_index += 1;
                if continuation.is_none() {
                    break;
                }
            }

            debug!("Found {count} Azure DevOps test suites");
            Ok(suites)
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsTestSuiteListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsTestSuiteListRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(AzureDevOpsTestSuiteListRequest<'static> => Vec<Arc<AzureDevOpsTestSuite>>, effects = [Read]);

cloud_terrastodon_registry_execution::register_runtime_test!(
    AzureDevOpsTestSuiteListRequest<'static>,
    name = "azure-devops-test-suite-list",
    cached
);
