use crate::azure_devops_rest::authenticate_azure_devops_request;
use crate::azure_devops_work_item_relation_change::AzureDevOpsWorkItemRelationChange;
use arbitrary::Arbitrary;
use cloud_terrastodon_azure_devops_types::AzureDevOpsJsonPatchOperation;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops_types::AzureDevOpsWorkItem;
use cloud_terrastodon_azure_devops_types::AzureDevOpsWorkItemId;
use cloud_terrastodon_azure_devops_types::AzureDevOpsWorkItemUrl;
use cloud_terrastodon_azure_types::ArbitraryJson;
use cloud_terrastodon_credentials::AzureDevOpsAuthContext;
use cloud_terrastodon_rest::RequestHeaders;
use cloud_terrastodon_rest::RestRequest;
use eyre::Result;
use eyre::ensure;
use reqwest::Method;
use reqwest::Url;
use std::borrow::Cow;
use std::future::Future;
use std::future::IntoFuture;
use std::pin::Pin;

#[derive(Debug, Clone, facet::Facet)]
pub struct AzureDevOpsWorkItemRelationChangeRequest<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    pub project: Option<AzureDevOpsProjectArgument<'a>>,
    pub auth_context: Cow<'a, AzureDevOpsAuthContext>,
    pub id: AzureDevOpsWorkItemId,
    pub change: AzureDevOpsWorkItemRelationChange,
    /// Indicate whether to validate the changes without saving the work item.
    pub validate_only: bool,
    /// Do not fire notifications for this change.
    pub suppress_notifications: bool,
    pub if_rev: Option<i32>,
}

impl<'a> Arbitrary<'a> for AzureDevOpsWorkItemRelationChangeRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            project: Option::<AzureDevOpsProjectArgument<'static>>::arbitrary(u)?,
            auth_context: Cow::Owned(AzureDevOpsAuthContext::None),
            id: AzureDevOpsWorkItemId::arbitrary(u)?,
            change: AzureDevOpsWorkItemRelationChange::arbitrary(u)?,
            validate_only: bool::arbitrary(u)?,
            suppress_notifications: bool::arbitrary(u)?,
            if_rev: Option::<i32>::arbitrary(u)?,
        })
    }
}

impl<'a> AzureDevOpsWorkItemRelationChangeRequest<'a> {
    fn item_url(&self) -> Result<Url> {
        AzureDevOpsWorkItemUrl::new(self.org_url.clone(), self.id)
            .into_url_with_project(self.project.clone())
    }
}

impl<'a> IntoFuture for AzureDevOpsWorkItemRelationChangeRequest<'a> {
    type Output = Result<ArbitraryJson>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + 'a>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let mut get_url = self.item_url()?;
            get_url.query_pairs_mut().append_pair("$expand", "all");
            let get_request = RestRequest::new(Method::GET, get_url.as_str())?;
            let item: AzureDevOpsWorkItem =
                authenticate_azure_devops_request(get_request, self.auth_context.as_ref())?
                    .receive()
                    .await?;
            ensure!(
                item.id == self.id,
                "Work item response did not match the requested ID"
            );
            if let Some(revision) = self.if_rev {
                ensure!(revision == item.rev, "Work item revision has changed");
            }

            let mut patch = self.change.patch(self.org_url.as_ref(), &item)?;
            ensure!(item.rev > 0, "Revision must be positive");
            patch.insert(
                0,
                AzureDevOpsJsonPatchOperation::Test {
                    path: "/rev".to_owned(),
                    value: ArbitraryJson::try_from_facet(&item.rev)?,
                },
            );

            let mut patch_url = self.item_url()?;
            patch_url.set_query(None);
            patch_url
                .query_pairs_mut()
                .append_pair("api-version", "7.1")
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
            let patch_request = RestRequest::new(Method::PATCH, patch_url.as_str())?
                .body(facet_json::to_string(&patch)?)
                .headers(RequestHeaders::from_header(
                    "Content-Type",
                    "application/json-patch+json",
                )?);
            authenticate_azure_devops_request(patch_request, self.auth_context.as_ref())?
                .receive()
                .await
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsWorkItemRelationChangeRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsWorkItemRelationChangeRequest<'static>);
cloud_terrastodon_registry::register_into_future!(
    AzureDevOpsWorkItemRelationChangeRequest<'static> => ArbitraryJson,
    effects = [Write]
);
