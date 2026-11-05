use arbitrary::Arbitrary;
use cloud_terrastodon_auth::AzureDevOpsAuthContext;
use cloud_terrastodon_azure_devops_types::AzureDevOpsCollectionResponse;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops_types::AzureDevOpsWorkItemQuery;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_pathing::sanitize_windows_path_component;
use cloud_terrastodon_rest::RestRequest;
use cloud_terrastodon_rest::RestService;
use reqwest::Method;
use std::borrow::Cow;
use std::path::PathBuf;

#[derive(Debug, Clone, facet::Facet)]
#[facet(traits(Clone))]
pub struct WorkItemQueriesForProjectRequest<'a> {
    #[facet(skip_serializing)]
    pub cancellation: CancellationScope,

    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    pub auth_context: Cow<'a, AzureDevOpsAuthContext>,
    pub project: AzureDevOpsProjectArgument<'a>,
}

pub fn fetch_queries_for_project<'a>(
    org_url: &'a AzureDevOpsOrganizationUrl,
    project: impl Into<AzureDevOpsProjectArgument<'a>>,
    auth_context: &'a AzureDevOpsAuthContext,
    cancellation: &CancellationScope,
) -> WorkItemQueriesForProjectRequest<'a> {
    WorkItemQueriesForProjectRequest {
        cancellation: cancellation.clone(),
        org_url: Cow::Borrowed(org_url),
        auth_context: Cow::Borrowed(auth_context),
        project: project.into(),
    }
}

impl<'a> Arbitrary<'a> for WorkItemQueriesForProjectRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            cancellation: CancellationScope::arbitrary(u)?,
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            auth_context: Cow::Owned(AzureDevOpsAuthContext::None),
            project: AzureDevOpsProjectArgument::arbitrary(u)?.into_owned(),
        })
    }
}

#[async_trait]
impl<'a> CacheableCommand for WorkItemQueriesForProjectRequest<'a> {
    type Output = Vec<AzureDevOpsWorkItemQuery>;

    fn cache_key(&self) -> CacheKey {
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
        CacheKey::new(path)
    }

    async fn run(self) -> eyre::Result<Self::Output> {
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
            &self.cancellation,
        )?
        .cache(self.cache_key())
        .azure_devops_auth_context(self.auth_context.as_ref())?
        .receive::<AzureDevOpsCollectionResponse<Vec<AzureDevOpsWorkItemQuery>>>()
        .await?;
        Ok(response.value)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(WorkItemQueriesForProjectRequest<'a>, 'a);
cloud_terrastodon_registry::register_thing!(WorkItemQueriesForProjectRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(WorkItemQueriesForProjectRequest<'static>);
cloud_terrastodon_registry_execution::register_into_future!(WorkItemQueriesForProjectRequest<'static> => Vec<AzureDevOpsWorkItemQuery>, effects = [Read]);

#[cfg(test)]
mod test {
    use crate::fetch_queries_for_project;
    use crate::get_default_organization_url;
    use crate::get_default_project_name;
    use cloud_terrastodon_auth::AzureDevOpsAuthContext;
    use cloud_terrastodon_azure_devops_types::AzureDevOpsWorkItemQuery;
    use cloud_terrastodon_command::CancellationToken;

    #[test]
    fn project_cache_scopes_preserve_selector_identity_and_name_matching() -> eyre::Result<()> {
        use crate::fetch_all_azure_devops_service_endpoints;
        use crate::fetch_azure_devops_team_members;
        use crate::fetch_azure_devops_teams_for_project;
        use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
        use cloud_terrastodon_azure_devops_types::AzureDevOpsProjectArgument;
        use cloud_terrastodon_azure_devops_types::AzureDevOpsProjectId;
        use cloud_terrastodon_azure_devops_types::AzureDevOpsProjectName;
        use cloud_terrastodon_azure_devops_types::AzureDevOpsTeamId;
        use cloud_terrastodon_command::CacheableCommand;
        use cloud_terrastodon_command::CancellationScope;
        use std::path::PathBuf;

        let org: AzureDevOpsOrganizationUrl = "synthetic".parse()?;
        let team: AzureDevOpsTeamId = "11111111-1111-1111-1111-111111111111".parse()?;
        let auth = AzureDevOpsAuthContext::None;
        let scope = CancellationScope::unbound();
        let keys = |project: AzureDevOpsProjectArgument<'static>| -> [PathBuf; 4] {
            [
                fetch_queries_for_project(&org, project.clone(), &auth, &scope)
                    .cache_key()
                    .path,
                fetch_azure_devops_teams_for_project(&org, project.clone(), &auth, &scope)
                    .cache_key()
                    .path,
                fetch_azure_devops_team_members(&org, project.clone(), &team, &auth, &scope)
                    .cache_key()
                    .path,
                fetch_all_azure_devops_service_endpoints(&org, project, &auth, &scope)
                    .cache_key()
                    .path,
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

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        CancellationToken::new()
            .with_async_scope(|cancellation| async move {
                let azure_devops_auth_context = AzureDevOpsAuthContext::for_tenant(
                    &cloud_terrastodon_auth::AuthContext::explicit_azure_cli(),
                    crate::test_tenant_id::test_tenant_id()?,
                )?;
                let org_url = get_default_organization_url(&cancellation).await?;
                let project_name = get_default_project_name(&cancellation).await?;
                let queries = fetch_queries_for_project(
                    &org_url,
                    &project_name,
                    &azure_devops_auth_context,
                    &cancellation,
                )
                .await?;
                let flattened = AzureDevOpsWorkItemQuery::flatten_many(&queries);
                assert!(
                    !flattened.is_empty(),
                    "Expected at least one Azure DevOps work item query"
                );
                assert!(
                    flattened.iter().all(|entry| !entry.child.name.is_empty()),
                    "Expected Azure DevOps work item queries to include names"
                );
                Ok(())
            })
            .await
    }
}
