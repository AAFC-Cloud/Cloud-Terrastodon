use crate::WorkItemListResponse;
use crate::azure_devops_rest::authenticate_azure_devops_request;
use arbitrary::Arbitrary;
use chrono::DateTime;
use chrono::Utc;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops_types::AzureDevOpsWorkItem;
use cloud_terrastodon_azure_devops_types::AzureDevOpsWorkItemExpand;
use cloud_terrastodon_azure_devops_types::AzureDevOpsWorkItemId;
use cloud_terrastodon_credentials::AzureDevOpsAuthContext;
use cloud_terrastodon_rest::RestRequest;
use eyre::Result;
use eyre::ensure;
use facet::Facet;
use reqwest::Method;
use reqwest::Url;
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::future::Future;
use std::future::IntoFuture;
use std::pin::Pin;

#[derive(Debug, Clone, Facet)]
pub struct AzureDevOpsWorkItemsGetRequest<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    pub project: Option<AzureDevOpsProjectArgument<'a>>,
    pub auth_context: Cow<'a, AzureDevOpsAuthContext>,
    pub ids: Vec<AzureDevOpsWorkItemId>,
    pub expand: AzureDevOpsWorkItemExpand,
    pub fields: Vec<String>,
    /// Read the work item state as of this UTC date-time snapshot.
    ///
    /// Azure DevOps expects an ISO 8601 date-time string, for example
    /// `2017-12-21T19:42:54.230Z`. See the official [`List Work Items`](https://learn.microsoft.com/en-us/rest/api/azure/devops/wit/work-items/list?view=azure-devops-rest-7.1)
    /// documentation for the `asOf` query parameter.
    pub as_of: Option<DateTime<Utc>>,
}

impl<'a> Arbitrary<'a> for AzureDevOpsWorkItemsGetRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            project: Some(AzureDevOpsProjectArgument::arbitrary(u)?.into_owned()),
            auth_context: Cow::Owned(AzureDevOpsAuthContext::None),
            ids: Vec::<AzureDevOpsWorkItemId>::arbitrary(u)?,
            expand: AzureDevOpsWorkItemExpand::arbitrary(u)?,
            fields: Vec::<String>::arbitrary(u)?,
            as_of: Option::<DateTime<Utc>>::arbitrary(u)?,
        })
    }
}

impl<'a> IntoFuture for AzureDevOpsWorkItemsGetRequest<'a> {
    type Output = Result<Vec<AzureDevOpsWorkItem>>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + 'a>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            ensure!(!self.ids.is_empty(), "Specify at least one work item ID");
            let mut seen = BTreeSet::new();
            let ids: Vec<_> = self
                .ids
                .iter()
                .copied()
                .filter(|id| seen.insert(*id))
                .collect();
            let mut items = BTreeMap::new();
            for chunk in ids.chunks(200) {
                #[derive(Facet)]
                #[facet(rename_all = "camelCase")]
                struct Body {
                    ids: Vec<AzureDevOpsWorkItemId>,
                    #[facet(rename = "$expand")]
                    expand: cloud_terrastodon_azure_devops_types::AzureDevOpsWorkItemExpand,
                    #[facet(skip_serializing_if = Vec::is_empty)]
                    fields: Vec<String>,
                    #[facet(skip_serializing_if = Option::is_none)]
                    as_of: Option<String>,
                    error_policy: String,
                }

                let body = Body {
                    ids: chunk.to_vec(),
                    expand: self.expand,
                    fields: self.fields.clone(),
                    as_of: self.as_of.as_ref().map(DateTime::to_rfc3339),
                    error_policy: "fail".to_owned(),
                };
                let mut url = Url::parse(&self.org_url.to_string())?;
                {
                    let mut path = url
                        .path_segments_mut()
                        .map_err(|_| eyre::eyre!("Invalid organization URL"))?;
                    path.pop_if_empty();
                    if let Some(project) = &self.project {
                        path.push(&project.to_string());
                    }
                    path.extend(["_apis", "wit", "workitemsbatch"]);
                }
                url.query_pairs_mut().append_pair("api-version", "7.1");
                let request = RestRequest::new(Method::POST, url.as_str())?
                    .body(facet_json::to_string(&body)?);
                let response: WorkItemListResponse<AzureDevOpsWorkItem> =
                    authenticate_azure_devops_request(request, self.auth_context.as_ref())?
                        .receive()
                        .await?;
                for item in response.value {
                    ensure!(
                        seen.contains(&item.id),
                        "Batch response contained an unrequested work item"
                    );
                    ensure!(
                        items.insert(item.id, item).is_none(),
                        "Batch response contained a duplicate work item"
                    );
                }
            }
            ids.into_iter()
                .map(|id| {
                    items.remove(&id).ok_or_else(|| {
                        eyre::eyre!("A requested work item was missing from the response")
                    })
                })
                .collect()
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsWorkItemsGetRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsWorkItemsGetRequest<'static>);
cloud_terrastodon_registry::register_into_future!(
    AzureDevOpsWorkItemsGetRequest<'static> => Vec<AzureDevOpsWorkItem>,
    effects = [Read]
);
