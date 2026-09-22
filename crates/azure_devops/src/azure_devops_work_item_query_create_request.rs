use crate::azure_devops_rest::authenticate_azure_devops_request;
use arbitrary::Arbitrary;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops_types::AzureDevOpsWorkItemQuery;
use cloud_terrastodon_azure_devops_types::AzureDevOpsWorkItemQueryPath;
use cloud_terrastodon_credentials::AzureDevOpsAuthContext;
use cloud_terrastodon_rest::RestRequest;
use eyre::Result;
use eyre::ensure;
use facet::Facet;
use reqwest::Method;
use std::borrow::Cow;
use std::future::Future;
use std::future::IntoFuture;
use std::pin::Pin;

#[derive(Debug, Clone, Facet)]
pub struct AzureDevOpsWorkItemQueryCreateRequest<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    pub project: AzureDevOpsProjectArgument<'a>,
    pub auth_context: Cow<'a, AzureDevOpsAuthContext>,
    pub folder: AzureDevOpsWorkItemQueryPath,
    pub name: String,
    pub wiql: String,
}

impl<'a> Arbitrary<'a> for AzureDevOpsWorkItemQueryCreateRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            project: AzureDevOpsProjectArgument::arbitrary(u)?.into_owned(),
            auth_context: Cow::Owned(AzureDevOpsAuthContext::None),
            folder: AzureDevOpsWorkItemQueryPath::arbitrary(u)?,
            name: String::arbitrary(u)?,
            wiql: String::arbitrary(u)?,
        })
    }
}

impl<'a> IntoFuture for AzureDevOpsWorkItemQueryCreateRequest<'a> {
    type Output = Result<AzureDevOpsWorkItemQuery>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + 'a>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            ensure!(
                !self.name.trim().is_empty() && !self.wiql.trim().is_empty(),
                "Query name and WIQL must not be empty"
            );
            #[derive(Facet)]
            struct Body {
                name: String,
                wiql: String,
            }
            // Example: https://dev.azure.com/{organization}/{project}/_apis/wit/queries/Shared%20Queries/Example?api-version=7.1
            let url = format!(
                "{}/{}/_apis/wit/queries/{}?api-version=7.1",
                self.org_url, self.project, self.folder
            );
            let request =
                RestRequest::new(Method::POST, &url)?.body(facet_json::to_string(&Body {
                    name: self.name,
                    wiql: self.wiql,
                })?);
            authenticate_azure_devops_request(request, self.auth_context.as_ref())?
                .receive()
                .await
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsWorkItemQueryCreateRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsWorkItemQueryCreateRequest<'static>);
cloud_terrastodon_registry::register_into_future!(
    AzureDevOpsWorkItemQueryCreateRequest<'static> => AzureDevOpsWorkItemQuery,
    effects = [Write]
);
