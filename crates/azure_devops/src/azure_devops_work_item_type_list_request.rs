use crate::WorkItemListResponse;
use crate::azure_devops_rest::authenticate_azure_devops_request;
use arbitrary::Arbitrary;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops_types::AzureDevOpsWorkItemTypeDefinition;
use cloud_terrastodon_credentials::AzureDevOpsAuthContext;
use cloud_terrastodon_rest::RestRequest;
use eyre::Result;
use facet::Facet;
use reqwest::Method;
use reqwest::Url;
use std::borrow::Cow;
use std::future::Future;
use std::future::IntoFuture;
use std::pin::Pin;

#[derive(Debug, Clone, Facet)]
pub struct AzureDevOpsWorkItemTypeListRequest<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    pub project: AzureDevOpsProjectArgument<'a>,
    pub auth_context: Cow<'a, AzureDevOpsAuthContext>,
}

impl<'a> Arbitrary<'a> for AzureDevOpsWorkItemTypeListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            project: AzureDevOpsProjectArgument::arbitrary(u)?.into_owned(),
            auth_context: Cow::Owned(AzureDevOpsAuthContext::None),
        })
    }
}

impl<'a> IntoFuture for AzureDevOpsWorkItemTypeListRequest<'a> {
    type Output = Result<Vec<AzureDevOpsWorkItemTypeDefinition>>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + 'a>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let project = self.project.to_string();
            let mut url = Url::parse(&self.org_url.to_string())?;
            url.path_segments_mut()
                .map_err(|_| eyre::eyre!("Invalid organization URL"))?
                .pop_if_empty()
                .push(&project)
                .extend(["_apis", "wit", "workitemtypes"]);
            url.query_pairs_mut().append_pair("api-version", "7.1");
            authenticate_azure_devops_request(
                RestRequest::new(Method::GET, url.as_str())?,
                self.auth_context.as_ref(),
            )?
            .receive::<WorkItemListResponse<AzureDevOpsWorkItemTypeDefinition>>()
            .await
            .map(|response| response.value)
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsWorkItemTypeListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsWorkItemTypeListRequest<'static>);
cloud_terrastodon_registry::register_into_future!(
    AzureDevOpsWorkItemTypeListRequest<'static> => Vec<AzureDevOpsWorkItemTypeDefinition>,
    effects = [Read]
);
