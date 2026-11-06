use crate::WorkItemListResponse;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth::AzureDevOpsAuthContext;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops_types::AzureDevOpsWorkItemQuery;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_rest::MicrosoftContinuationToken;
use cloud_terrastodon_rest::RestRequest;
use eyre::ensure;
use facet::Facet;
use reqwest::Method;
use reqwest::Url;
use std::borrow::Cow;

#[derive(Debug, Clone, Facet)]
#[facet(traits(Clone))]
pub struct AzureDevOpsWorkItemQueryListRequest<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    pub project: AzureDevOpsProjectArgument<'a>,
    pub auth_context: Cow<'a, AzureDevOpsAuthContext>,
    pub depth: u32,
}

impl<'a> Arbitrary<'a> for AzureDevOpsWorkItemQueryListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            project: AzureDevOpsProjectArgument::arbitrary(u)?.into_owned(),
            auth_context: Cow::Owned(AzureDevOpsAuthContext::None),
            depth: u.arbitrary()?,
        })
    }
}

impl<'a> ExecutableRequest for AzureDevOpsWorkItemQueryListRequest<'a> {
    type Error = eyre::Report;
    type Output = Vec<AzureDevOpsWorkItemQuery>;

    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            ensure!(
                self.depth <= 2,
                "Query expansion depth must be between zero and two"
            );

            let mut base_url = Url::parse(&self.org_url.to_string())?;
            base_url
                .path_segments_mut()
                .map_err(|_| eyre::eyre!("Invalid organization URL"))?
                .pop_if_empty()
                .push(&self.project.to_string())
                .extend(["_apis", "wit", "queries"]);
            base_url
                .query_pairs_mut()
                .append_pair("api-version", "7.1")
                .append_pair("$expand", "all")
                .append_pair("$depth", &self.depth.to_string());

            let mut continuation: Option<MicrosoftContinuationToken> = None;
            let mut seen = std::collections::BTreeSet::new();
            let mut queries = Vec::new();
            loop {
                executor.cancellation().bail_if_cancelled()?;
                let mut url = base_url.clone();
                if let Some(token) = &continuation {
                    url.query_pairs_mut()
                        .append_pair("continuationToken", token.as_str());
                }
                let request = RestRequest::from_method_and_url(Method::GET, url)?
                    .azure_devops_auth_context(self.auth_context.as_ref())?;
                let (response, next): (WorkItemListResponse<AzureDevOpsWorkItemQuery>, _) = request
                    .receive_with_ms_continuation_token(&executor)
                    .await?;
                queries.extend(response.value);
                let Some(token) = next else {
                    break;
                };
                ensure!(
                    seen.insert(token.clone()),
                    "Query listing returned a repeated continuation token"
                );
                continuation = Some(token);
            }
            Ok(queries)
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsWorkItemQueryListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsWorkItemQueryListRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(
    AzureDevOpsWorkItemQueryListRequest<'static> => Vec<AzureDevOpsWorkItemQuery>,
    effects = [Read]
);
