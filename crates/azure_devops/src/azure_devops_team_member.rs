use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsTeamId;
use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsTeamMember;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_command::impl_cacheable_into_future;
use std::path::PathBuf;
use tracing::debug;

pub struct AzureDevOpsTeamMembersRequest<'a> {
    org_url: &'a AzureDevOpsOrganizationUrl,
    project: AzureDevOpsProjectArgument<'a>,
    team_id: &'a AzureDevOpsTeamId,
}

pub fn fetch_azure_devops_team_members<'a>(
    org_url: &'a AzureDevOpsOrganizationUrl,
    project: impl Into<AzureDevOpsProjectArgument<'a>>,
    team_id: &'a AzureDevOpsTeamId,
) -> AzureDevOpsTeamMembersRequest<'a> {
    AzureDevOpsTeamMembersRequest {
        org_url,
        project: project.into(),
        team_id,
    }
}

#[async_trait]
impl<'a> CacheableCommand for AzureDevOpsTeamMembersRequest<'a> {
    type Output = Vec<AzureDevOpsTeamMember>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "devops",
            "team",
            "list-member",
            "--project",
            &self.project.to_string(),
            "--team",
            &self.team_id.to_string(),
        ]))
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        let project = self.project;
        let team_id = self.team_id;
        debug!("Fetching Azure DevOps teams for project {project}");
        let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
        cmd.args([
            "devops",
            "team",
            "list-member",
            "--organization",
            self.org_url.to_string().as_str(),
            "--team",
            &team_id.to_string(),
            "--project",
            &project.to_string(),
            "--output",
            "json",
        ]);
        cmd.cache(CacheKey::new(PathBuf::from_iter([
            "az",
            "devops",
            "team",
            "list-member",
            "--project",
            &project.to_string(),
            "--team",
            &team_id.to_string(),
        ])));

        let response = cmd.run::<Vec<AzureDevOpsTeamMember>>().await?;
        debug!(
            "Found {} Azure DevOps team members for project {project} team {team_id:?}",
            response.len()
        );
        Ok(response)
    }
}

impl_cacheable_into_future!(AzureDevOpsTeamMembersRequest<'a>, 'a);

#[cfg(test)]
mod test {
    use crate::prelude::fetch_all_azure_devops_projects;
    use crate::prelude::fetch_azure_devops_team_members;
    use crate::prelude::fetch_azure_devops_teams_for_project;
    use crate::prelude::get_default_organization_url;

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

        println!("{team:?}");
        let members = fetch_azure_devops_team_members(&org_url, &project.id, &team.id).await?;

        assert!(
            !members.is_empty(),
            "Expected at least one member in the team"
        );
        for member in members {
            println!("Found member: {member:?}");
        }
        Ok(())
    }
}
