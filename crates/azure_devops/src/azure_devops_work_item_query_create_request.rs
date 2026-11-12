use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureDevOpsAuthSelection;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops_types::AzureDevOpsWorkItemQuery;
use cloud_terrastodon_azure_devops_types::AzureDevOpsWorkItemQueryPath;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_rest::RestRequest;
use eyre::ensure;
use facet::Facet;
use reqwest::Method;
use std::borrow::Cow;

#[derive(Debug, Clone, Facet)]
#[facet(traits(Clone))]
pub struct AzureDevOpsWorkItemQueryCreateRequest<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    pub project: AzureDevOpsProjectArgument<'a>,
    pub auth_selection: AzureDevOpsAuthSelection,
    pub folder: AzureDevOpsWorkItemQueryPath,
    pub name: String,
    pub wiql: String,
}

impl<'a> Arbitrary<'a> for AzureDevOpsWorkItemQueryCreateRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            project: AzureDevOpsProjectArgument::arbitrary(u)?.into_owned(),
            auth_selection: AzureDevOpsAuthSelection::arbitrary(u)?,
            folder: AzureDevOpsWorkItemQueryPath::arbitrary(u)?,
            name: String::arbitrary(u)?,
            wiql: String::arbitrary(u)?,
        })
    }
}

impl<'a> ExecutableRequest for AzureDevOpsWorkItemQueryCreateRequest<'a> {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> Result<CloudTerrastodonExecutor, Self::Error> {
        executor.validate_authentication_target(
            &self.auth_selection.into(),
            &self.org_url.api_url("dev.azure.com", "_apis", &[])?,
        )?;
        executor.with_authentication_lease(&self.auth_selection.into())
    }

    type Error = eyre::Report;
    type Output = AzureDevOpsWorkItemQuery;

    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
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
            request
                .authentication(self.auth_selection.into())
                .receive(&executor)
                .await
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsWorkItemQueryCreateRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsWorkItemQueryCreateRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(
    AzureDevOpsWorkItemQueryCreateRequest<'static> => AzureDevOpsWorkItemQuery,
    effects = [Write]
);
