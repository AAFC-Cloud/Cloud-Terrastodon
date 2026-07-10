use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops_types::AzureDevOpsTeamId;
use cloud_terrastodon_azure_devops_types::AzureDevOpsTeamMember;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::async_trait;
use std::borrow::Cow;
use std::path::PathBuf;
use tracing::debug;

#[derive(Debug, Clone, facet::Facet)]
pub struct AzureDevOpsTeamMembersRequest<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    pub project: AzureDevOpsProjectArgument<'a>,
    pub team_id: Cow<'a, AzureDevOpsTeamId>,
}

pub fn fetch_azure_devops_team_members<'a>(
    org_url: &'a AzureDevOpsOrganizationUrl,
    project: impl Into<AzureDevOpsProjectArgument<'a>>,
    team_id: &'a AzureDevOpsTeamId,
) -> AzureDevOpsTeamMembersRequest<'a> {
    AzureDevOpsTeamMembersRequest {
        org_url: Cow::Borrowed(org_url),
        project: project.into(),
        team_id: Cow::Borrowed(team_id),
    }
}

impl<'a> Arbitrary<'a> for AzureDevOpsTeamMembersRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            project: AzureDevOpsProjectArgument::arbitrary(u)?.into_owned(),
            team_id: Cow::Owned(AzureDevOpsTeamId::arbitrary(u)?),
        })
    }
}

#[async_trait]
impl<'a> CacheableCommand for AzureDevOpsTeamMembersRequest<'a> {
    type Output = Vec<AzureDevOpsTeamMember>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "devops",
            self.org_url.organization_name.as_ref(),
            "team",
            "list-member",
            "--project",
            &self.project.to_string(),
            "--team",
            &self.team_id.to_string(),
        ]))
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        debug!("Fetching Azure DevOps teams for project {}", self.project);
        let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
        cmd.args([
            "devops",
            "team",
            "list-member",
            "--organization",
            self.org_url.to_string().as_str(),
            "--team",
            &self.team_id.to_string(),
            "--project",
            &self.project.to_string(),
            "--output",
            "json",
        ]);
        cmd.cache(self.cache_key());

        let response = cmd.run::<Vec<AzureDevOpsTeamMember>>().await?;
        debug!(
            "Found {} Azure DevOps team members for project {} team {}",
            response.len(),
            self.project,
            self.team_id
        );
        Ok(response)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(AzureDevOpsTeamMembersRequest<'a>, 'a);
cloud_terrastodon_registry::register_thing!(AzureDevOpsTeamMembersRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsTeamMembersRequest<'static>);
cloud_terrastodon_registry::register_into_future!(AzureDevOpsTeamMembersRequest<'static> => Vec<AzureDevOpsTeamMember>, effects = [Read]);

#[cfg(test)]
mod test {
    use crate::fetch_all_azure_devops_projects;
    use crate::fetch_azure_devops_team_members;
    use crate::fetch_azure_devops_teams_for_project;
    use crate::get_default_organization_url;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let org_url = get_default_organization_url().await?;
        let project = fetch_all_azure_devops_projects(&org_url)
            .await?
            .into_iter()
            .next()
            .unwrap();
        let teams = fetch_azure_devops_teams_for_project(&org_url, &project.id).await?;
        let team = teams
            .into_iter()
            .next()
            .expect("Expected at least one team in the project");

        assert!(!team.name.is_empty());
        let members = fetch_azure_devops_team_members(&org_url, &project.id, &team.id).await?;

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
    }
}
use arbitrary::Arbitrary;
