use arbitrary::Arbitrary;
use cloud_terrastodon_auth::AzureDevOpsAuthContext;
use cloud_terrastodon_azure_devops_types::AzureDevOpsCollectionResponse;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops_types::AzureDevOpsTeam;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_execution::RequestExecution;
use cloud_terrastodon_pathing::sanitize_windows_path_component;
use cloud_terrastodon_rest::RestRequest;
use cloud_terrastodon_rest::RestService;
use reqwest::Method;
use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Clone, facet::Facet)]
#[facet(traits(Clone))]
pub struct AzureDevOpsTeamsForProjectRequest<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    pub auth_context: Cow<'a, AzureDevOpsAuthContext>,
    pub project: AzureDevOpsProjectArgument<'a>,
}

#[track_caller]
pub fn fetch_azure_devops_teams_for_project<'a>(
    org_url: &'a AzureDevOpsOrganizationUrl,
    project: impl Into<AzureDevOpsProjectArgument<'a>>,
    auth_context: &'a AzureDevOpsAuthContext,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, AzureDevOpsTeamsForProjectRequest<'a>> {
    executor.execute(AzureDevOpsTeamsForProjectRequest {
        org_url: Cow::Borrowed(org_url),
        auth_context: Cow::Borrowed(auth_context),
        project: project.into(),
    })
}

impl<'a> Arbitrary<'a> for AzureDevOpsTeamsForProjectRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            auth_context: Cow::Owned(AzureDevOpsAuthContext::None),
            project: AzureDevOpsProjectArgument::arbitrary(u)?.into_owned(),
        })
    }
}
impl<'a> HasCacheKey for AzureDevOpsTeamsForProjectRequest<'a> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        let mut path = PathBuf::from_iter(["az", "devops"]);
        if self.org_url.collection_name().is_some() {
            path.push("server");
            path.push(self.org_url.instance_scheme());
            for segment in self.org_url.instance_authority_and_path().split('/') {
                path.push(sanitize_windows_path_component(segment));
            }
            path.push("collection");
        }
        path.push(sanitize_windows_path_component(self.org_url.name()));
        path.push("team");
        path.push("list");
        path.push("project");
        match &self.project {
            AzureDevOpsProjectArgument::Id(id) => {
                path.push("id");
                path.push(id.to_string());
            }
            AzureDevOpsProjectArgument::Name(name) => {
                path.push("name");
                path.push(sanitize_windows_path_component(
                    &name.to_string().to_ascii_lowercase(),
                ));
            }
        }
        CacheKey::from_relative_path(path)
    }
}

impl<'a> InvalidatesOwnCacheKey for AzureDevOpsTeamsForProjectRequest<'a> {}

impl<'a> ExecutableRequest for AzureDevOpsTeamsForProjectRequest<'a> {
    type Error = eyre::Report;
    type Output = Vec<Arc<AzureDevOpsTeam>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            // The Core API uses $top/$skip, not a continuation header:
            // https://learn.microsoft.com/en-us/rest/api/azure/devops/core/teams/get-teams?view=azure-devops-rest-7.1
            let cache_key = self.cache_key()?;
            let mut values = Vec::new();
            let mut offset = 0usize;
            loop {
                let mut url = self.org_url.api_url(
                    "dev.azure.com",
                    "",
                    &[
                        ("api-version", "7.1"),
                        ("$top", "100"),
                        ("$skip", &offset.to_string()),
                    ],
                )?;
                url.path_segments_mut()
                    .map_err(|()| eyre::eyre!("organization URL must have a hierarchical path"))?
                    .pop_if_empty()
                    .extend(["_apis", "projects", &self.project.to_string(), "teams"]);
                let response = RestRequest::from_method_and_url_for_service(
                    Method::GET,
                    url,
                    RestService::AzureDevOps,
                )?
                .cache(cache_key.join("pages")?.join(offset.to_string())?)
                .azure_devops_auth_context(self.auth_context.as_ref())?
                .receive_cached::<AzureDevOpsCollectionResponse<Vec<Arc<AzureDevOpsTeam>>>>(
                    &executor,
                    CacheOperationOwner::of::<AzureDevOpsTeamsForProjectRequest<'static>>(),
                )
                .await?;
                executor.cancellation().bail_if_cancelled()?;
                if response.value.is_empty() {
                    break;
                }
                offset = offset
                    .checked_add(response.value.len())
                    .ok_or_else(|| eyre::eyre!("team page offset overflow"))?;
                values.extend(response.value.iter().cloned());
            }
            Ok(values)
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsTeamsForProjectRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsTeamsForProjectRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(AzureDevOpsTeamsForProjectRequest<'static> => Vec<Arc<AzureDevOpsTeam>>, effects = [Read]);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fetch_all_azure_devops_projects;
    use crate::get_default_organization_url;
    use cloud_terrastodon_auth::AuthContext;
    use cloud_terrastodon_auth::AzureDevOpsAuthContext;
    use cloud_terrastodon_command::CancellationToken;

    #[tokio::test]
    async fn it_works() -> eyre::Result<()> {
        CancellationToken::new()
            .with_async_scope(|cancellation| async move {
                let auth_context = AuthContext::explicit_azure_cli();
                let azure_devops_auth_context = AzureDevOpsAuthContext::for_tenant(
                    &auth_context,
                    crate::test_tenant_id::test_tenant_id()?,
                )?;
                let executor =
                    crate::test_executor::test_executor(&azure_devops_auth_context, &cancellation)?;
                let org_url = get_default_organization_url(&cancellation).await?;
                let project = fetch_all_azure_devops_projects(
                    &org_url,
                    &azure_devops_auth_context,
                    &executor,
                )
                .await?
                .into_iter()
                .next()
                .unwrap();
                let results = fetch_azure_devops_teams_for_project(
                    &org_url,
                    &project.id,
                    &azure_devops_auth_context,
                    &executor,
                )
                .await?;
                assert!(!results.is_empty());
                assert!(results.iter().all(|team| !team.name.is_empty()));
                Ok(())
            })
            .await
    }
}
