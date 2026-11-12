use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureDevOpsAuthSelection;
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
use cloud_terrastodon_rest::RestRequest;
use facet::Facet;
use reqwest::Method;
use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::debug;

#[derive(Debug, Clone, Facet)]
#[facet(traits(Clone))]
pub struct AzureDevOpsTestSuiteCasesListRequest<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    pub project: AzureDevOpsProjectArgument<'a>,
    pub plan: String,
    pub suite: String,
    pub auth_selection: AzureDevOpsAuthSelection,
}

impl<'a> Arbitrary<'a> for AzureDevOpsTestSuiteCasesListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            project: AzureDevOpsProjectArgument::arbitrary(u)?.into_owned(),
            plan: String::arbitrary(u)?,
            suite: String::arbitrary(u)?,
            auth_selection: AzureDevOpsAuthSelection::arbitrary(u)?,
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
    type Output = Arc<Vec<SuiteTestCase>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            debug!("Fetching Azure DevOps test cases for suite");

            #[derive(Facet)]
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
                .authentication(self.auth_selection.into())
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

cloud_terrastodon_registry_execution::register_runtime_test!(
    AzureDevOpsTestSuiteCasesListRequest<'static>,
    name = "azure-devops-test-suite-cases-list",
    cached
);
