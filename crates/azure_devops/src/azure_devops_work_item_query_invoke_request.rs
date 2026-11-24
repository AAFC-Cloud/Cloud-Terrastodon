use crate::azure_devops_work_item_query_selector::AzureDevOpsWorkItemQuerySelector;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureDevOpsAuthSelection;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops_types::WorkItemQueryResult;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_rest::RestRequest;
use eyre::Result;
use eyre::WrapErr;
use eyre::ensure;
use facet::Facet;
use reqwest::Method;
use reqwest::Url;
use std::borrow::Cow;
use std::num::NonZeroU32;
use std::sync::Arc;

/// Executes a saved or inline [WIQL query (7.1)](https://learn.microsoft.com/en-us/rest/api/azure/devops/wit/wiql/query-by-wiql?view=azure-devops-rest-7.1).
#[derive(Debug, Clone, Facet)]
#[facet(traits(Clone))]
pub struct AzureDevOpsWorkItemQueryInvokeRequest<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    pub project: Option<AzureDevOpsProjectArgument<'a>>,
    pub auth_selection: AzureDevOpsAuthSelection,
    pub query: AzureDevOpsWorkItemQuerySelector,
    /// Maximum result count, sent as the API's `$top` query parameter.
    pub top: Option<NonZeroU32>,
}

impl<'a> Arbitrary<'a> for AzureDevOpsWorkItemQueryInvokeRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            project: Option::<AzureDevOpsProjectArgument<'static>>::arbitrary(u)?,
            auth_selection: AzureDevOpsAuthSelection::arbitrary(u)?,
            query: AzureDevOpsWorkItemQuerySelector::arbitrary(u)?,
            top: bool::arbitrary(u)?
                .then(|| {
                    u.int_in_range(0..=(i32::MAX as u32 - 1))
                        .map(|offset| NonZeroU32::MIN.saturating_add(offset))
                })
                .transpose()?,
        })
    }
}

impl AzureDevOpsWorkItemQueryInvokeRequest<'_> {
    fn into_rest_request(self) -> Result<RestRequest> {
        let mut url = Url::parse(&self.org_url.to_string())?;
        {
            let mut path = url
                .path_segments_mut()
                .map_err(|_| eyre::eyre!("Invalid organization URL"))?;
            path.pop_if_empty();
            if let Some(project) = &self.project {
                path.push(&project.to_string());
            }
            path.extend(["_apis", "wit"]);
        }
        url.query_pairs_mut().append_pair("api-version", "7.1");
        if let Some(top) = self.top {
            let top = i32::try_from(top.get()).wrap_err("The WIQL result limit exceeds int32")?;
            url.query_pairs_mut().append_pair("$top", &top.to_string());
        }

        let (method, body) = match self.query {
            AzureDevOpsWorkItemQuerySelector::Id(id) => {
                url.path_segments_mut()
                    .map_err(|_| eyre::eyre!("Invalid organization URL"))?
                    .push("wiql")
                    .push(&id.to_string());
                (Method::GET, None)
            }
            AzureDevOpsWorkItemQuerySelector::Wiql(query) => {
                ensure!(!query.trim().is_empty(), "WIQL must not be empty");
                #[derive(Facet)]
                struct Body {
                    query: String,
                }
                url.path_segments_mut()
                    .map_err(|_| eyre::eyre!("Invalid organization URL"))?
                    .push("wiql");
                (Method::POST, Some(facet_json::to_string(&Body { query })?))
            }
        };

        let mut request = RestRequest::from_method_and_url(method, url)?
            .cancellation_safe()
            .authentication(self.auth_selection.into());
        request.body = body;
        Ok(request)
    }

    /// Reuse a query under the owning inventory's invalidation scope.
    pub(crate) async fn receive_cached(
        self,
        executor: &CloudTerrastodonExecutor,
        key: CacheKey,
    ) -> Result<Arc<WorkItemQueryResult>> {
        self.into_rest_request()?
            .cache(key)
            .receive_cached(
                executor,
                CacheOperationOwner::of::<AzureDevOpsWorkItemQueryInvokeRequest<'static>>(),
            )
            .await
    }
}

impl<'a> ExecutableRequest for AzureDevOpsWorkItemQueryInvokeRequest<'a> {
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
    type Output = WorkItemQueryResult;

    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move { self.into_rest_request()?.receive(&executor).await })
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsWorkItemQueryInvokeRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsWorkItemQueryInvokeRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(
    AzureDevOpsWorkItemQueryInvokeRequest<'static> => WorkItemQueryResult,
    effects = [Read]
);
