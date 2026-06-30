use cloud_terrastodon_azure_devops::AzureDevOpsGroup;
use cloud_terrastodon_azure_devops::AzureDevOpsProject;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsTeam;
use cloud_terrastodon_azure_devops::fetch_all_azure_devops_projects;
use cloud_terrastodon_azure_devops::fetch_azure_devops_group_members_v2;
use cloud_terrastodon_azure_devops::fetch_azure_devops_groups_for_project;
use cloud_terrastodon_azure_devops::fetch_azure_devops_teams_for_project;
use cloud_terrastodon_azure_devops::get_default_organization_url;
use cloud_terrastodon_command::ParallelFallibleWorkQueue;
use cloud_terrastodon_command::to_writer_pretty;
use eyre::Result;
use eyre::bail;
use facet_json::RawJson;
use std::io::stdout;
use tracing::Instrument;
use tracing::info;
use tracing::info_span;

/// Azure DevOps project dump command.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureDevOpsProjectDumpArgs {
    /// Project id (UUID) or project name.
    #[facet(opaque, proxy = String)]
    project: AzureDevOpsProjectArgument<'static>,
}

#[derive(facet::Facet)]
struct AzureDevOpsProjectDumpPayload {
    project: AzureDevOpsProject,
    teams: Vec<AzureDevOpsTeam>,
    groups: Vec<AzureDevOpsGroup>,
    group_members: Vec<RawJson<'static>>,
}

impl AzureDevOpsProjectDumpArgs {
    /// Stubbed invoke implementation for the `dump` command.
    pub async fn invoke(self) -> Result<()> {
        let span = info_span!("azure_devops_project_dump", project=%self.project);
        let _guard = span.clone().entered();

        info!("Fetching projects");
        let org_url = get_default_organization_url()
            .into_future()
            .instrument(span.clone())
            .await?;
        let projects = fetch_all_azure_devops_projects(&org_url)
            .into_future()
            .instrument(span.clone())
            .await?;

        let Some(project) = projects.into_iter().find(|p| self.project.matches(p)) else {
            bail!("No project found matching '{}'.", self.project);
        };

        let teams = fetch_azure_devops_teams_for_project(&org_url, &project)
            .into_future()
            .instrument(span.clone())
            .await?;

        let groups = fetch_azure_devops_groups_for_project(&org_url, &project)
            .into_future()
            .instrument(span.clone())
            .await?;

        let mut group_members = ParallelFallibleWorkQueue::new("fetching group members", 4);
        for group in groups.iter() {
            let org_url = org_url.clone();
            let group_descriptor = group.descriptor.clone();
            let span = span.clone();
            group_members.enqueue(async move {
                fetch_azure_devops_group_members_v2(&org_url, &group_descriptor)
                    .into_future()
                    .instrument(span.clone())
                    .await
            });
        }
        let group_members = group_members.join().await?;

        let payload = AzureDevOpsProjectDumpPayload {
            project,
            teams,
            groups,
            group_members,
        };

        to_writer_pretty(stdout(), &payload)?;

        Ok(())
    }
}
