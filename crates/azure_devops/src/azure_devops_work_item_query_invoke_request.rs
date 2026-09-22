use crate::azure_devops_rest::authenticate_azure_devops_request;
use crate::azure_devops_work_item_query_selector::AzureDevOpsWorkItemQuerySelector;
use arbitrary::Arbitrary;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops_types::WorkItemQueryResult;
use cloud_terrastodon_credentials::AzureDevOpsAuthContext;
use cloud_terrastodon_rest::RestRequest;
use eyre::Result;
use eyre::ensure;
use facet::Facet;
use reqwest::Method;
use reqwest::Url;
use std::borrow::Cow;
use std::future::Future;
use std::future::IntoFuture;
use std::pin::Pin;

#[derive(Debug, Clone, Facet)]
pub struct AzureDevOpsWorkItemQueryInvokeRequest<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    pub project: Option<AzureDevOpsProjectArgument<'a>>,
    pub auth_context: Cow<'a, AzureDevOpsAuthContext>,
    pub query: AzureDevOpsWorkItemQuerySelector,
}

impl<'a> Arbitrary<'a> for AzureDevOpsWorkItemQueryInvokeRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            project: Option::<AzureDevOpsProjectArgument<'static>>::arbitrary(u)?,
            auth_context: Cow::Owned(AzureDevOpsAuthContext::None),
            query: AzureDevOpsWorkItemQuerySelector::arbitrary(u)?,
        })
    }
}

impl<'a> IntoFuture for AzureDevOpsWorkItemQueryInvokeRequest<'a> {
    type Output = Result<WorkItemQueryResult>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + 'a>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
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

            let mut request = RestRequest::new(method, url.as_str())?;
            request.body = body;
            authenticate_azure_devops_request(request, self.auth_context.as_ref())?
                .receive()
                .await
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsWorkItemQueryInvokeRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsWorkItemQueryInvokeRequest<'static>);
cloud_terrastodon_registry::register_into_future!(
    AzureDevOpsWorkItemQueryInvokeRequest<'static> => WorkItemQueryResult,
    effects = [Read]
);
