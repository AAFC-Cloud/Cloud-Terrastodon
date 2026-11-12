use super::azure_devops_project_dump_payload::AzureDevOpsProjectDumpPayload;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureDevOpsAuthenticationArgs;
use cloud_terrastodon_azure_devops::AzureDevOpsGroupMembersV2Request;
use cloud_terrastodon_azure_devops::AzureDevOpsGroupsListRequest;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectsListRequest;
use cloud_terrastodon_azure_devops::AzureDevOpsTeamsForProjectRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ConcurrentIteratorExt;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use eyre::bail;
use facet::Facet;
use std::borrow::Cow;
use std::num::NonZeroUsize;
use tracing::Instrument;
use tracing::info;
use tracing::info_span;

/// Azure DevOps project dump command.
#[derive(Facet, Debug, Clone)]
pub struct AzureDevOpsProjectDumpArgs {
    #[facet(flatten)]
    pub authentication: AzureDevOpsAuthenticationArgs,

    /// Azure DevOps organization name or URL.
    #[facet(figue::named)]
    pub org: AzureDevOpsOrganizationUrl,
    /// Project id (UUID) or project name.
    #[facet(figue::positional, proxy = String)]
    project: AzureDevOpsProjectArgument<'static>,
}

impl AzureDevOpsProjectDumpArgs {
    /// Stubbed invoke implementation for the `dump` command.
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let span = info_span!("azure_devops_project_dump", project=%self.project);
        let _guard = span.clone().entered();

        info!("Fetching projects");
        let auth_selection = self.authentication.resolve(&self.org, executor).await?;
        let projects = AzureDevOpsProjectsListRequest {
            org_url: Cow::Borrowed(&self.org),
            auth_selection,
        }
        .execute(executor)
        .into_future()
        .instrument(span.clone())
        .await?;

        let Some(project) = projects.iter().find(|p| self.project.matches(p)).cloned() else {
            bail!("No project found matching '{}'.", self.project);
        };

        let teams = AzureDevOpsTeamsForProjectRequest {
            org_url: Cow::Borrowed(&self.org),
            auth_selection,
            project: (project.as_ref()).into(),
        }
        .execute(executor)
        .into_future()
        .instrument(span.clone())
        .await?;

        let groups = AzureDevOpsGroupsListRequest {
            org_url: Cow::Borrowed(&self.org),
            project: (project.as_ref()).into(),
            auth_selection,
        }
        .execute(executor)
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
                    let members = AzureDevOpsGroupMembersV2Request {
                        org_url: Cow::Borrowed(&self.org),
                        group_id: Cow::Borrowed(&group.descriptor),
                        auth_selection,
                    }
                    .execute(&child)
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

        Ok(CliOutput::facet(payload))
    }
}
