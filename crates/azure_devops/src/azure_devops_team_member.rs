use arbitrary::Arbitrary;
use cloud_terrastodon_auth::AzureDevOpsAuthContext;
use cloud_terrastodon_azure_devops_types::AzureDevOpsCollectionResponse;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops_types::AzureDevOpsTeamId;
use cloud_terrastodon_azure_devops_types::AzureDevOpsTeamMember;
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
pub struct AzureDevOpsTeamMembersRequest<'a> {
    #[facet(skip_serializing)]
    pub cancellation: CancellationScope,

    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    pub auth_context: Cow<'a, AzureDevOpsAuthContext>,
    pub project: AzureDevOpsProjectArgument<'a>,
    pub team_id: Cow<'a, AzureDevOpsTeamId>,
}

pub fn fetch_azure_devops_team_members<'a>(
    org_url: &'a AzureDevOpsOrganizationUrl,
    project: impl Into<AzureDevOpsProjectArgument<'a>>,
    team_id: &'a AzureDevOpsTeamId,
    auth_context: &'a AzureDevOpsAuthContext,
    cancellation: &CancellationScope,
) -> AzureDevOpsTeamMembersRequest<'a> {
    AzureDevOpsTeamMembersRequest {
        cancellation: cancellation.clone(),
        org_url: Cow::Borrowed(org_url),
        auth_context: Cow::Borrowed(auth_context),
        project: project.into(),
        team_id: Cow::Borrowed(team_id),
    }
}

impl<'a> Arbitrary<'a> for AzureDevOpsTeamMembersRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            cancellation: CancellationScope::arbitrary(u)?,
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            auth_context: Cow::Owned(AzureDevOpsAuthContext::None),
            project: AzureDevOpsProjectArgument::arbitrary(u)?.into_owned(),
            team_id: Cow::Owned(AzureDevOpsTeamId::arbitrary(u)?),
        })
    }
}

#[async_trait]
impl<'a> CacheableCommand for AzureDevOpsTeamMembersRequest<'a> {
    type Output = Vec<AzureDevOpsTeamMember>;

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
        path.push("team");
        path.push("list-member");
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
        path.push("team");
        path.push("id");
        path.push(sanitize_windows_path_component(&self.team_id.to_string()));
        CacheKey::new(path)
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        // The Core API uses $top/$skip, not a continuation header:
        // https://learn.microsoft.com/en-us/rest/api/azure/devops/core/teams/get-team-members-with-extended-properties?view=azure-devops-rest-7.1
        let cache_key = self.cache_key();
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
                .extend([
                    "_apis",
                    "projects",
                    &self.project.to_string(),
                    "teams",
                    &self.team_id.to_string(),
                    "members",
                ]);
            let response = RestRequest::from_method_and_url_for_service(
                Method::GET,
                url,
                RestService::AzureDevOps,
                &self.cancellation,
            )?
            .cache(cache_key.join("pages").join(offset.to_string()))
            .azure_devops_auth_context(self.auth_context.as_ref())?
            .receive::<AzureDevOpsCollectionResponse<Vec<AzureDevOpsTeamMember>>>()
            .await?;
            self.cancellation.bail_if_cancelled()?;
            if response.value.is_empty() {
                break;
            }
            offset = offset
                .checked_add(response.value.len())
                .ok_or_else(|| eyre::eyre!("team page offset overflow"))?;
            values.extend(response.value);
        }
        Ok(values)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(AzureDevOpsTeamMembersRequest<'a>, 'a);
cloud_terrastodon_registry::register_thing!(AzureDevOpsTeamMembersRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsTeamMembersRequest<'static>);
cloud_terrastodon_registry_execution::register_into_future!(AzureDevOpsTeamMembersRequest<'static> => Vec<AzureDevOpsTeamMember>, effects = [Read]);

#[cfg(test)]
mod test {
    use crate::fetch_all_azure_devops_projects;
    use crate::fetch_azure_devops_team_members;
    use crate::fetch_azure_devops_teams_for_project;
    use crate::get_default_organization_url;
    use cloud_terrastodon_auth::AuthContext;
    use cloud_terrastodon_auth::AzureDevOpsAuthContext;
    use cloud_terrastodon_command::CancellationToken;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        CancellationToken::new()
            .with_async_scope(|cancellation| async move {
                let auth_context = AuthContext::explicit_azure_cli();
                let azure_devops_auth_context = AzureDevOpsAuthContext::for_tenant(
                    &auth_context,
                    crate::test_tenant_id::test_tenant_id()?,
                )?;
                let org_url = get_default_organization_url(&cancellation).await?;
                let project = fetch_all_azure_devops_projects(
                    &org_url,
                    &azure_devops_auth_context,
                    &cancellation,
                )
                .await?
                .into_iter()
                .next()
                .unwrap();
                let teams = fetch_azure_devops_teams_for_project(
                    &org_url,
                    &project.id,
                    &azure_devops_auth_context,
                    &cancellation,
                )
                .await?;
                let team = teams
                    .into_iter()
                    .next()
                    .expect("Expected at least one team in the project");

                assert!(!team.name.is_empty());
                let members = fetch_azure_devops_team_members(
                    &org_url,
                    &project.id,
                    &team.id,
                    &azure_devops_auth_context,
                    &cancellation,
                )
                .await?;

                assert!(
                    !members.is_empty(),
                    "Expected at least one member in the team"
                );
                assert!(
                    members
                        .iter()
                        .all(|member| !member.identity.display_name.is_empty())
                );
                Ok(())
            })
            .await
    }
}
