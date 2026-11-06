use arbitrary::Arbitrary;
use chrono::DateTime;
use chrono::Utc;
use cloud_terrastodon_auth::AzureDevOpsAuthContext;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops_types::AzureDevOpsWorkItem;
use cloud_terrastodon_azure_devops_types::AzureDevOpsWorkItemExpand;
use cloud_terrastodon_azure_devops_types::AzureDevOpsWorkItemId;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_rest::RestRequest;
use eyre::ensure;
use facet::Facet;
use reqwest::Method;
use reqwest::Url;
use std::borrow::Cow;

#[derive(Debug, Clone, Facet)]
#[facet(traits(Clone))]
pub struct AzureDevOpsWorkItemGetRequest<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    pub project: Option<AzureDevOpsProjectArgument<'a>>,
    pub auth_context: Cow<'a, AzureDevOpsAuthContext>,
    pub id: AzureDevOpsWorkItemId,
    pub expand: AzureDevOpsWorkItemExpand,
    pub fields: Vec<String>,
    /// Read the work item state as of this UTC date-time snapshot.
    ///
    /// Azure DevOps expects an ISO 8601 date-time string, for example
    /// `2017-12-21T19:42:54.230Z`. See the official [`Get Work Item`](https://learn.microsoft.com/en-us/rest/api/azure/devops/wit/work-items/get-work-item?view=azure-devops-rest-7.1)
    /// documentation for the `asOf` query parameter.
    pub as_of: Option<DateTime<Utc>>,
}

impl<'a> Arbitrary<'a> for AzureDevOpsWorkItemGetRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            project: Some(AzureDevOpsProjectArgument::arbitrary(u)?.into_owned()),
            auth_context: Cow::Owned(AzureDevOpsAuthContext::None),
            id: AzureDevOpsWorkItemId::arbitrary(u)?,
            expand: AzureDevOpsWorkItemExpand::arbitrary(u)?,
            fields: Vec::<String>::arbitrary(u)?,
            as_of: Option::<DateTime<Utc>>::arbitrary(u)?,
        })
    }
}

impl<'a> ExecutableRequest for AzureDevOpsWorkItemGetRequest<'a> {
    type Error = eyre::Report;
    type Output = AzureDevOpsWorkItem;

    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
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
                path.extend(["_apis", "wit", "workitems"])
                    .push(&self.id.to_string());
            }
            url.query_pairs_mut()
                .append_pair("api-version", "7.1")
                .append_pair("$expand", self.expand.as_str());
            if !self.fields.is_empty() {
                url.query_pairs_mut()
                    .append_pair("fields", &self.fields.join(","));
            }
            if let Some(as_of) = &self.as_of {
                url.query_pairs_mut()
                    .append_pair("asOf", &as_of.to_rfc3339());
            }
            let id = self.id;
            let request = RestRequest::from_method_and_url(Method::GET, url)?;
            let item: AzureDevOpsWorkItem = request
                .azure_devops_auth_context(self.auth_context.as_ref())?
                .receive(&executor)
                .await?;
            ensure!(
                item.id == id,
                "Work item response did not match the requested ID"
            );
            Ok(item)
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsWorkItemGetRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsWorkItemGetRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(
    AzureDevOpsWorkItemGetRequest<'static> => AzureDevOpsWorkItem,
    effects = [Read]
);
