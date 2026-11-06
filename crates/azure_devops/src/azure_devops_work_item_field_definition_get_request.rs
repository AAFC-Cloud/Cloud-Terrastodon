use arbitrary::Arbitrary;
use cloud_terrastodon_auth::AzureDevOpsAuthContext;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsWorkItemFieldDefinition;
use cloud_terrastodon_azure_devops_types::AzureDevOpsWorkItemFieldName;
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
pub struct AzureDevOpsWorkItemFieldDefinitionGetRequest<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    pub auth_context: Cow<'a, AzureDevOpsAuthContext>,
    pub name: AzureDevOpsWorkItemFieldName,
}

impl<'a> Arbitrary<'a> for AzureDevOpsWorkItemFieldDefinitionGetRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            auth_context: Cow::Owned(AzureDevOpsAuthContext::None),
            name: AzureDevOpsWorkItemFieldName::arbitrary(u)?,
        })
    }
}

impl<'a> ExecutableRequest for AzureDevOpsWorkItemFieldDefinitionGetRequest<'a> {
    type Error = eyre::Report;
    type Output = AzureDevOpsWorkItemFieldDefinition;

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
                .extend(["_apis", "wit", "fields"])
                .push(self.name.as_ref());
            url.query_pairs_mut().append_pair("api-version", "7.1");
            let request = RestRequest::from_method_and_url(Method::GET, url)?
                .azure_devops_auth_context(self.auth_context.as_ref())?;
            request.receive(&executor).await
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsWorkItemFieldDefinitionGetRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(
    AzureDevOpsWorkItemFieldDefinitionGetRequest<'static>
);
cloud_terrastodon_registry_execution::register_executable_request!(
    AzureDevOpsWorkItemFieldDefinitionGetRequest<'static> => AzureDevOpsWorkItemFieldDefinition,
    effects = [Read]
);
