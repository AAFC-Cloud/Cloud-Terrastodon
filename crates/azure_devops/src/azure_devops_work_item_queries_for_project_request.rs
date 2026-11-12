use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureDevOpsAuthSelection;
use cloud_terrastodon_azure_devops_types::AzureDevOpsCollectionResponse;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops_types::AzureDevOpsWorkItemQuery;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_pathing::sanitize_windows_path_component;
use cloud_terrastodon_rest::RestRequest;
use cloud_terrastodon_rest::RestService;
use facet::Facet;
use reqwest::Method;
use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Clone, Facet)]
#[facet(traits(Clone))]
pub struct AzureDevOpsWorkItemQueriesForProjectRequest<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    pub auth_selection: AzureDevOpsAuthSelection,
    pub project: AzureDevOpsProjectArgument<'a>,
}

impl<'a> Arbitrary<'a> for AzureDevOpsWorkItemQueriesForProjectRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            auth_selection: AzureDevOpsAuthSelection::arbitrary(u)?,
            project: AzureDevOpsProjectArgument::arbitrary(u)?.into_owned(),
        })
    }
}
impl<'a> HasCacheKey for AzureDevOpsWorkItemQueriesForProjectRequest<'a> {
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
        path.push("query");
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

impl<'a> InvalidatesOwnCacheKey for AzureDevOpsWorkItemQueriesForProjectRequest<'a> {}

impl<'a> ExecutableRequest for AzureDevOpsWorkItemQueriesForProjectRequest<'a> {
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
    type Output = Arc<Vec<AzureDevOpsWorkItemQuery>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            // https://learn.microsoft.com/en-us/rest/api/azure/devops/wit/queries/list?view=azure-devops-rest-7.1
            let mut url = self.org_url.api_url(
                "dev.azure.com",
                "",
                &[("api-version", "7.1"), ("$expand", "all"), ("$depth", "2")],
            )?;
            url.path_segments_mut()
                .map_err(|()| eyre::eyre!("organization URL must have a hierarchical path"))?
                .pop_if_empty()
                .extend([&self.project.to_string(), "_apis", "wit", "queries"]);
            let response = RestRequest::from_method_and_url_for_service(
                Method::GET,
                url,
                RestService::AzureDevOps,
            )?
            .cache(self.cache_key()?)
            .authentication(self.auth_selection.into())
            .receive_cached::<AzureDevOpsCollectionResponse<Arc<Vec<AzureDevOpsWorkItemQuery>>>>(
                &executor,
                CacheOperationOwner::of::<AzureDevOpsWorkItemQueriesForProjectRequest<'static>>(),
            )
            .await?;
            Ok(Arc::clone(&response.value))
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsWorkItemQueriesForProjectRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(
    AzureDevOpsWorkItemQueriesForProjectRequest<'static>
);
cloud_terrastodon_registry_execution::register_executable_request!(AzureDevOpsWorkItemQueriesForProjectRequest<'static> => Arc<Vec<AzureDevOpsWorkItemQuery>>, effects = [Read]);

#[cfg(test)]
mod test {
    use crate::AzureDevOpsWorkItemQueriesForProjectRequest;
    use std::borrow::Cow;

    #[test]
    fn project_cache_scopes_preserve_selector_identity_and_name_matching() -> eyre::Result<()> {
        use crate::AzureDevOpsServiceEndpointsListRequest;
        use crate::AzureDevOpsTeamMembersRequest;
        use crate::AzureDevOpsTeamsForProjectRequest;
        use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
        use cloud_terrastodon_azure_devops_types::AzureDevOpsProjectArgument;
        use cloud_terrastodon_azure_devops_types::AzureDevOpsProjectId;
        use cloud_terrastodon_azure_devops_types::AzureDevOpsProjectName;
        use cloud_terrastodon_azure_devops_types::AzureDevOpsTeamId;
        use cloud_terrastodon_cache::HasCacheKey;
        use std::path::PathBuf;

        let org: AzureDevOpsOrganizationUrl = "synthetic".parse()?;
        let team: AzureDevOpsTeamId = "11111111-1111-1111-1111-111111111111".parse()?;
        let auth_selection = crate::test_executor::authentication_selection();
        let keys = |project: AzureDevOpsProjectArgument<'static>| -> [PathBuf; 4] {
            [
                AzureDevOpsWorkItemQueriesForProjectRequest {
                    org_url: Cow::Borrowed(&org),
                    auth_selection,
                    project: (project.clone()),
                }
                .cache_key()
                .unwrap()
                .scope
                .components()
                .iter()
                .collect::<PathBuf>(),
                AzureDevOpsTeamsForProjectRequest {
                    org_url: Cow::Borrowed(&org),
                    auth_selection,
                    project: (project.clone()),
                }
                .cache_key()
                .unwrap()
                .scope
                .components()
                .iter()
                .collect::<PathBuf>(),
                AzureDevOpsTeamMembersRequest {
                    org_url: Cow::Borrowed(&org),
                    auth_selection,
                    project: (project.clone()),
                    team_id: Cow::Borrowed(&team),
                }
                .cache_key()
                .unwrap()
                .scope
                .components()
                .iter()
                .collect::<PathBuf>(),
                AzureDevOpsServiceEndpointsListRequest {
                    org_url: Cow::Borrowed(&org),
                    auth_selection,
                    project: (project),
                }
                .cache_key()
                .unwrap()
                .scope
                .components()
                .iter()
                .collect::<PathBuf>(),
            ]
        };
        let named = |value: &str| -> eyre::Result<AzureDevOpsProjectArgument<'static>> {
            Ok(AzureDevOpsProjectName::try_new(value)?.into())
        };
        let id: AzureDevOpsProjectId = "11111111-1111-1111-1111-111111111111".parse()?;
        for (id_key, name_key) in keys(id.into())
            .into_iter()
            .zip(keys(named("11111111-1111-1111-1111-111111111111")?))
        {
            assert_ne!(
                id_key, name_key,
                "an unresolved name must not invalidate or replace an ID scope"
            );
        }
        assert_eq!(
            keys(named("Sample Project")?),
            keys(named("sample project")?)
        );
        for (spaced, compact) in keys(named("Sample Project")?)
            .into_iter()
            .zip(keys(named("SampleProject")?))
        {
            assert_ne!(spaced, compact);
        }
        for (spaced, escaped) in keys(named("Sample Project")?)
            .into_iter()
            .zip(keys(named("Sample%20Project")?))
        {
            assert_ne!(
                spaced, escaped,
                "literal percent escapes must remain distinguishable"
            );
        }
        Ok(())
    }
}

cloud_terrastodon_registry_execution::register_runtime_test!(
    AzureDevOpsWorkItemQueriesForProjectRequest<'static>,
    name = "azure-devops-work-item-queries-for-project",
    cached
);
