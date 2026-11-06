use crate::WorkItemListResponse;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth::AzureDevOpsAuthContext;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsWorkItemFieldDefinition;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_rest::RestRequest;
use facet::Facet;
use reqwest::Method;
use reqwest::Url;
use std::borrow::Cow;

#[derive(Debug, Clone, Facet)]
#[facet(traits(Clone))]
pub struct AzureDevOpsWorkItemFieldDefinitionListRequest<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    pub auth_context: Cow<'a, AzureDevOpsAuthContext>,
}

impl<'a> Arbitrary<'a> for AzureDevOpsWorkItemFieldDefinitionListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            auth_context: Cow::Owned(AzureDevOpsAuthContext::None),
        })
    }
}

impl<'a> ExecutableRequest for AzureDevOpsWorkItemFieldDefinitionListRequest<'a> {
    type Error = eyre::Report;
    type Output = Vec<AzureDevOpsWorkItemFieldDefinition>;

    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let mut url = Url::parse(&self.org_url.to_string())?;
            url.path_segments_mut()
                .map_err(|_| eyre::eyre!("Invalid organization URL"))?
                .pop_if_empty()
                .extend(["_apis", "wit", "fields"]);
            url.query_pairs_mut().append_pair("api-version", "7.1");
            let request = RestRequest::from_method_and_url(Method::GET, url)?
                .azure_devops_auth_context(self.auth_context.as_ref())?;
            request
                .receive::<WorkItemListResponse<AzureDevOpsWorkItemFieldDefinition>>(&executor)
                .await
                .map(|response| response.value)
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsWorkItemFieldDefinitionListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(
    AzureDevOpsWorkItemFieldDefinitionListRequest<'static>
);
cloud_terrastodon_registry_execution::register_executable_request!(
    AzureDevOpsWorkItemFieldDefinitionListRequest<'static> => Vec<AzureDevOpsWorkItemFieldDefinition>,
    effects = [Read]
);
