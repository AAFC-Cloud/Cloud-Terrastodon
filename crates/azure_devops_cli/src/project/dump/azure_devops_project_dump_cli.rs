use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_azure::AzureDevOpsTenantArgumentExt;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsGroup;
use cloud_terrastodon_azure_devops::AzureDevOpsGroupMembersV2Response;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProject;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsTeam;
use cloud_terrastodon_azure_devops::fetch_all_azure_devops_projects;
use cloud_terrastodon_azure_devops::fetch_azure_devops_group_members_v2;
use cloud_terrastodon_azure_devops::fetch_azure_devops_groups_for_project;
use cloud_terrastodon_azure_devops::fetch_azure_devops_teams_for_project;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ConcurrentIteratorExt;
use eyre::Result;
use eyre::bail;
use facet::Facet;
use std::io::stdout;
use std::num::NonZeroUsize;
use std::sync::Arc;
use tracing::Instrument;
use tracing::info;
use tracing::info_span;

/// Azure DevOps project dump command.
#[derive(Facet, Debug, Clone)]
pub struct AzureDevOpsProjectDumpArgs {
    /// Tenant id or tracked alias for bearer authentication; PAT authentication may omit it.
    #[facet(figue::named)]
    pub tenant: Option<AzureTenantArgument<'static>>,

    /// Azure DevOps organization name or URL.
    #[facet(figue::named)]
    pub org: AzureDevOpsOrganizationUrl,
    /// Project id (UUID) or project name.
    #[facet(figue::positional, proxy = String)]
    project: AzureDevOpsProjectArgument<'static>,
}

#[derive(Facet)]
struct AzureDevOpsProjectDumpPayload {
    project: Arc<AzureDevOpsProject>,
    teams: Vec<Arc<AzureDevOpsTeam>>,
    groups: Vec<Arc<AzureDevOpsGroup>>,
    group_members: Vec<Arc<AzureDevOpsGroupMembersV2Response>>,
}

impl AzureDevOpsProjectDumpArgs {
    /// Stubbed invoke implementation for the `dump` command.
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        let cancellation = executor.cancellation();
        let span = info_span!("azure_devops_project_dump", project=%self.project);
        let _guard = span.clone().entered();

        info!("Fetching projects");
        let azure_devops_auth_context = self
            .tenant
            .bind_auth_context(auth_context, cancellation)
            .await?;
        let executor = &azure_devops_auth_context.bind_executor(executor)?;
        let projects = fetch_all_azure_devops_projects(
            &self.org,
            azure_devops_auth_context.selection()?,
            executor,
        )
        .into_future()
        .instrument(span.clone())
        .await?;

        let Some(project) = projects.iter().find(|p| self.project.matches(p)).cloned() else {
            bail!("No project found matching '{}'.", self.project);
        };

        let teams = fetch_azure_devops_teams_for_project(
            &self.org,
            project.as_ref(),
            &azure_devops_auth_context,
            executor,
        )
        .into_future()
        .instrument(span.clone())
        .await?;

        let groups = fetch_azure_devops_groups_for_project(
            &self.org,
            project.as_ref(),
            &azure_devops_auth_context,
            executor,
        )
        .into_future()
        .instrument(span.clone())
        .await?;

        // Drain the complete batch before reporting an ordinary service error.
        let group_members = groups
            .iter()
            .map_concurrent(
                executor,
                NonZeroUsize::MIN.saturating_add(3),
                async |child, group| {
                    let members = fetch_azure_devops_group_members_v2(
                        &self.org,
                        &group.descriptor,
                        &azure_devops_auth_context,
                        &child,
                    )
                    .into_future()
                    .instrument(span.clone())
                    .await;
                    info!("Finished fetching group members");
                    eyre::Ok(members)
                },
            )
            .await?
            .into_iter()
            .collect::<Result<Vec<_>>>()?;

        let payload = AzureDevOpsProjectDumpPayload {
            project,
            teams,
            groups,
            group_members,
        };

        facet_json::to_writer_std_pretty(stdout(), &payload)
            .map_err(|error| eyre::eyre!("{error:?}"))?;

        Ok(())
    }
}
