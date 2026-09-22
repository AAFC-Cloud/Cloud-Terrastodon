use crate::WorkItemListResponse;
use crate::azure_devops_rest::authenticate_azure_devops_request;
use arbitrary::Arbitrary;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsWorkItemRelationType;
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
pub struct AzureDevOpsWorkItemRelationTypeListRequest<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    pub auth_context: Cow<'a, AzureDevOpsAuthContext>,
}

impl<'a> Arbitrary<'a> for AzureDevOpsWorkItemRelationTypeListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            auth_context: Cow::Owned(AzureDevOpsAuthContext::None),
        })
    }
}

impl<'a> IntoFuture for AzureDevOpsWorkItemRelationTypeListRequest<'a> {
    type Output = Result<Vec<AzureDevOpsWorkItemRelationType>>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + 'a>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let mut url = Url::parse(&self.org_url.to_string())?;
            url.path_segments_mut()
                .map_err(|_| eyre::eyre!("Invalid organization URL"))?
                .pop_if_empty()
                .extend(["_apis", "wit", "workitemrelationtypes"]);
            url.query_pairs_mut().append_pair("api-version", "7.1");
            authenticate_azure_devops_request(
                RestRequest::new(Method::GET, url.as_str())?,
                self.auth_context.as_ref(),
            )?
            .receive::<WorkItemListResponse<AzureDevOpsWorkItemRelationType>>()
            .await
            .map(|response| response.value)
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsWorkItemRelationTypeListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(
    AzureDevOpsWorkItemRelationTypeListRequest<'static>
);
cloud_terrastodon_registry::register_into_future!(
    AzureDevOpsWorkItemRelationTypeListRequest<'static> => Vec<AzureDevOpsWorkItemRelationType>,
    effects = [Read]
);
