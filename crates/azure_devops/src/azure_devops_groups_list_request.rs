use crate::AzureDevOpsProjectsListRequest;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureDevOpsAuthSelection;
use cloud_terrastodon_azure_devops_types::AzureDevOpsGraphDescriptorResult;
use cloud_terrastodon_azure_devops_types::AzureDevOpsGroup;
use cloud_terrastodon_azure_devops_types::AzureDevOpsGroupListResponse;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsProjectArgument;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_pathing::sanitize_windows_path_component;
use cloud_terrastodon_rest::MicrosoftContinuationToken;
use cloud_terrastodon_rest::RestRequest;
use eyre::bail;
use eyre::ensure;
use facet::Facet;
use reqwest::Method;
use std::borrow::Cow;
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::debug;

/// Lists Azure DevOps Graph groups in a project's scope.
///
/// [Groups List (7.1)](https://learn.microsoft.com/en-us/rest/api/azure/devops/graph/groups/list?view=azure-devops-rest-7.1) uses the project scope's descriptor.
#[derive(Debug, Clone, Facet)]
#[facet(traits(Clone))]
pub struct AzureDevOpsGroupsListRequest<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    pub project: AzureDevOpsProjectArgument<'a>,
    pub auth_selection: AzureDevOpsAuthSelection,
}

impl<'a> Arbitrary<'a> for AzureDevOpsGroupsListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            project: AzureDevOpsProjectArgument::arbitrary(u)?.into_owned(),
            auth_selection: AzureDevOpsAuthSelection::arbitrary(u)?,
        })
    }
}
impl<'a> HasCacheKey for AzureDevOpsGroupsListRequest<'a> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        let (selector_kind, selector) = match &self.project {
            AzureDevOpsProjectArgument::Id(id) => ("id", id.to_string()),
            AzureDevOpsProjectArgument::Name(name) => {
                ("name", name.to_string().to_ascii_lowercase())
            }
        };
        let mut path = PathBuf::from("az/devops");
        if self.org_url.collection_name().is_some() {
            path.push("server");
            path.push(self.org_url.instance_scheme());
            for segment in self.org_url.instance_authority_and_path().split('/') {
                path.push(sanitize_windows_path_component(segment));
            }
        }
        path.extend([
            "org",
            &sanitize_windows_path_component(&self.org_url.name().to_ascii_lowercase()),
            "project",
            selector_kind,
            &sanitize_windows_path_component(&selector),
            "group/list",
        ]);
        CacheKey::from_relative_path(path)
    }
}

impl<'a> InvalidatesOwnCacheKey for AzureDevOpsGroupsListRequest<'a> {}

impl<'a> ExecutableRequest for AzureDevOpsGroupsListRequest<'a> {
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
    type Output = Vec<Arc<AzureDevOpsGroup>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let project = &self.project;
            debug!("Fetching Azure DevOps groups for project {project}");
            let cache_key = self.cache_key()?;
            let project_id = match &self.project {
                AzureDevOpsProjectArgument::Id(id) => **id,
                AzureDevOpsProjectArgument::Name(_) => {
                    let projects = AzureDevOpsProjectsListRequest {
                        org_url: Cow::Borrowed(self.org_url.as_ref()),
                        auth_selection: self.auth_selection,
                    }
                    .execute(&executor)
                    .await?;
                    let Some(project) = projects
                        .iter()
                        .find(|project| self.project.matches(project))
                    else {
                        bail!("No project found matching '{}'.", self.project);
                    };
                    project.id
                }
            };
            // The project UUID is its graph storage key. Resolve it before using
            // scopeDescriptor so the list never defaults to the organization.
            let url = self.org_url.api_url(
                "vssps.dev.azure.com",
                &format!("_apis/graph/descriptors/{project_id}"),
                &[("api-version", "7.1")],
            )?;
            let descriptor = RestRequest::from_method_and_url(Method::GET, url)?
                .cache(cache_key.join("descriptor")?)
                .authentication(self.auth_selection.into())
                .receive_cached::<AzureDevOpsGraphDescriptorResult>(
                    &executor,
                    CacheOperationOwner::of::<AzureDevOpsGroupsListRequest<'static>>(),
                )
                .await?
                .value
                .to_string();
            ensure!(
                !descriptor.trim().is_empty(),
                "Azure DevOps returned an empty descriptor for project {project_id}"
            );
            let groups = collect_graph_groups(
                &self.org_url,
                Some(&descriptor),
                self.auth_selection,
                &cache_key,
                CacheOperationOwner::of::<AzureDevOpsGroupsListRequest<'static>>(),
                &executor,
            )
            .await?;

            debug!(
                "Found {} Azure DevOps groups for project {}",
                groups.len(),
                project
            );
            Ok(groups)
        })
    }
}

pub(crate) async fn collect_graph_groups(
    organization: &AzureDevOpsOrganizationUrl,
    scope_descriptor: Option<&str>,
    authentication: AzureDevOpsAuthSelection,
    key: &CacheKey,
    owner: CacheOperationOwner,
    executor: &CloudTerrastodonExecutor,
) -> eyre::Result<Vec<Arc<AzureDevOpsGroup>>> {
    let pages = key.join("pages")?;
    let mut groups = Vec::new();
    let mut continuation: Option<MicrosoftContinuationToken> = None;
    let mut seen = BTreeSet::new();
    let mut page_index = 0;
    loop {
        executor.cancellation().bail_if_cancelled()?;
        let mut query = vec![("api-version", "7.1-preview.1")];
        if let Some(descriptor) = scope_descriptor {
            query.push(("scopeDescriptor", descriptor));
        }
        if let Some(token) = &continuation {
            query.push(("continuationToken", token.as_str()));
        }
        let url = organization.api_url("vssps.dev.azure.com", "_apis/graph/groups", &query)?;
        let (response, next_continuation) = RestRequest::from_method_and_url(Method::GET, url)?
            .cache(pages.join(page_index.to_string())?)
            .authentication(authentication.into())
            .receive_with_ms_continuation_token_cached::<AzureDevOpsGroupListResponse>(
                executor, owner,
            )
            .await?;
        groups.extend(response.value.iter().cloned());
        let Some(token) = next_continuation else {
            break;
        };
        ensure!(
            seen.insert(token.clone()),
            "Azure DevOps returned a repeated group continuation token"
        );
        continuation = Some(token);
        page_index += 1;
    }
    Ok(groups)
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsGroupsListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsGroupsListRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(
    AzureDevOpsGroupsListRequest<'static> => Vec<Arc<AzureDevOpsGroup>>,
    effects = [Read]
);

cloud_terrastodon_registry_execution::register_runtime_test!(
    AzureDevOpsGroupsListRequest<'static>,
    name = "azure-devops-groups-list",
    cached
);
