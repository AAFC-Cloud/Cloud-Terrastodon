use crate::azure_devops_rest::authenticate_azure_devops_request;
use arbitrary::Arbitrary;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops_types::AzureDevOpsWorkItemFieldName;
use cloud_terrastodon_azure_devops_types::AzureDevOpsWorkItemType;
use cloud_terrastodon_azure_devops_types::AzureDevOpsWorkItemTypeField;
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
pub struct AzureDevOpsWorkItemTypeFieldGetRequest<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    pub project: AzureDevOpsProjectArgument<'a>,
    pub auth_context: Cow<'a, AzureDevOpsAuthContext>,
    pub work_item_type: AzureDevOpsWorkItemType,
    pub field: AzureDevOpsWorkItemFieldName,
}

impl<'a> Arbitrary<'a> for AzureDevOpsWorkItemTypeFieldGetRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            project: AzureDevOpsProjectArgument::arbitrary(u)?.into_owned(),
            auth_context: Cow::Owned(AzureDevOpsAuthContext::None),
            work_item_type: AzureDevOpsWorkItemType::arbitrary(u)?,
            field: AzureDevOpsWorkItemFieldName::arbitrary(u)?,
        })
    }
}

impl<'a> IntoFuture for AzureDevOpsWorkItemTypeFieldGetRequest<'a> {
    type Output = Result<AzureDevOpsWorkItemTypeField>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + 'a>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let project = self.project.to_string();
            let mut url = Url::parse(&self.org_url.to_string())?;
            url.path_segments_mut()
                .map_err(|_| eyre::eyre!("Invalid organization URL"))?
                .pop_if_empty()
                .push(&project)
                .extend(["_apis", "wit", "workitemtypes"])
                .push(self.work_item_type.as_ref())
                .extend(["fields"])
                .push(self.field.as_ref());
            url.query_pairs_mut()
                .append_pair("api-version", "7.1")
                .append_pair("$expand", "all");
            authenticate_azure_devops_request(
                RestRequest::new(Method::GET, url.as_str())?,
                self.auth_context.as_ref(),
            )?
            .receive()
            .await
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsWorkItemTypeFieldGetRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsWorkItemTypeFieldGetRequest<'static>);
cloud_terrastodon_registry::register_into_future!(
    AzureDevOpsWorkItemTypeFieldGetRequest<'static> => AzureDevOpsWorkItemTypeField,
    effects = [Read]
);
