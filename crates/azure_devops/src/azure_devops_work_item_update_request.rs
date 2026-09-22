use crate::azure_devops_rest::authenticate_azure_devops_request;
use arbitrary::Arbitrary;
use cloud_terrastodon_azure_devops_types::AzureDevOpsJsonPatch;
use cloud_terrastodon_azure_devops_types::AzureDevOpsJsonPatchOperation;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops_types::AzureDevOpsWorkItemId;
use cloud_terrastodon_azure_types::ArbitraryJson;
use cloud_terrastodon_credentials::AzureDevOpsAuthContext;
use cloud_terrastodon_rest::RequestHeaders;
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
pub struct AzureDevOpsWorkItemUpdateRequest<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    pub project: Option<AzureDevOpsProjectArgument<'a>>,
    pub auth_context: Cow<'a, AzureDevOpsAuthContext>,
    pub id: AzureDevOpsWorkItemId,
    pub patch: AzureDevOpsJsonPatch,
    /// Indicate whether to validate the changes without saving the work item.
    pub validate_only: bool,
    /// Do not fire notifications for this change.
    pub suppress_notifications: bool,
    pub if_rev: Option<i32>,
}

impl<'a> Arbitrary<'a> for AzureDevOpsWorkItemUpdateRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            project: Some(AzureDevOpsProjectArgument::arbitrary(u)?.into_owned()),
            auth_context: Cow::Owned(AzureDevOpsAuthContext::None),
            id: AzureDevOpsWorkItemId::arbitrary(u)?,
            patch: AzureDevOpsJsonPatch::arbitrary(u)?,
            validate_only: bool::arbitrary(u)?,
            suppress_notifications: bool::arbitrary(u)?,
            if_rev: Option::<i32>::arbitrary(u)?,
        })
    }
}

impl<'a> IntoFuture for AzureDevOpsWorkItemUpdateRequest<'a> {
    type Output = Result<cloud_terrastodon_azure_types::ArbitraryJson>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + 'a>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let mut patch = self.patch;
            if let Some(revision) = self.if_rev {
                ensure!(revision > 0, "Revision must be positive");
                patch.insert(
                    0,
                    AzureDevOpsJsonPatchOperation::Test {
                        path: "/rev".to_owned(),
                        value: ArbitraryJson::try_from_facet(&revision)?,
                    },
                );
            }
            let mut url = Url::parse(&self.org_url.to_string())?;
            {
                let mut path = url
                    .path_segments_mut()
                    .map_err(|_| eyre::eyre!("Invalid organization URL"))?;
                path.pop_if_empty();
                if let Some(project) = &self.project {
                    path.push(&project.to_string());
                }
                path.extend(["_apis", "wit", "workitems"])
                    .push(&self.id.to_string());
            }
            url.query_pairs_mut().append_pair("api-version", "7.1");
            url.query_pairs_mut()
                .append_pair(
                    "validateOnly",
                    if self.validate_only { "true" } else { "false" },
                )
                .append_pair(
                    "suppressNotifications",
                    if self.suppress_notifications {
                        "true"
                    } else {
                        "false"
                    },
                )
                .append_pair("$expand", "all");
            let request = RestRequest::new(Method::PATCH, url.as_str())?
                .headers(RequestHeaders::from_header(
                    "Content-Type",
                    "application/json-patch+json",
                )?)
                .body(facet_json::to_string(&patch)?);
            authenticate_azure_devops_request(request, self.auth_context.as_ref())?
                .receive()
                .await
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsWorkItemUpdateRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsWorkItemUpdateRequest<'static>);
cloud_terrastodon_registry::register_into_future!(
    AzureDevOpsWorkItemUpdateRequest<'static> => cloud_terrastodon_azure_types::ArbitraryJson,
    effects = [Write]
);
