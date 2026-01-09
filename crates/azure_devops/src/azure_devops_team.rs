use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsTeam;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::async_trait;
use std::path::PathBuf;
use tracing::debug;

pub struct AzureDevOpsTeamsForProjectRequest<'a> {
    org_url: &'a AzureDevOpsOrganizationUrl,
    project: AzureDevOpsProjectArgument<'a>,
}

pub fn fetch_azure_devops_teams_for_project<'a>(
    org_url: &'a AzureDevOpsOrganizationUrl,
    project: impl Into<AzureDevOpsProjectArgument<'a>>,
) -> AzureDevOpsTeamsForProjectRequest<'a> {
    AzureDevOpsTeamsForProjectRequest {
        org_url,
        project: project.into(),
    }
}

#[async_trait]
impl<'a> CacheableCommand for AzureDevOpsTeamsForProjectRequest<'a> {
    type Output = Vec<AzureDevOpsTeam>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "devops",
            self.org_url.organization_name.as_ref(),
            "team",
            "list",
            "--project",
            &self.project.to_string(),
        ]))
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        debug!("Fetching Azure DevOps teams for project {}", self.project);
        let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
        cmd.args([
            "devops",
            "team",
            "list",
            "--organization",
            self.org_url.to_string().as_str(),
            "--output",
            "json",
            "--project",
            &self.project.to_string(),
        ]);
        cmd.cache(self.cache_key());

        let response = cmd.run::<Vec<AzureDevOpsTeam>>().await?;
        debug!(
            "Found {} Azure DevOps teams for project {}",
            response.len(),
            self.project
        );
        Ok(response)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(AzureDevOpsTeamsForProjectRequest<'a>, 'a);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::fetch_all_azure_devops_projects;
    use crate::prelude::get_default_organization_url;

    #[tokio::test]
    async fn it_works() -> eyre::Result<()> {
        let org_url = get_default_organization_url().await?;
        let project = fetch_all_azure_devops_projects(&org_url)
            .await?
            .into_iter()
            .next()
            .unwrap();
        let results = fetch_azure_devops_teams_for_project(&org_url, &project.id).await?;
        assert!(!results.is_empty());
        for value in results.iter().take(5) {
            println!("Found value: {value:#?}");
        }
        Ok(())
    }
}
