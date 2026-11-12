use super::azure_devops_agent_pool_summary::AzureDevOpsAgentPoolSummary;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureDevOpsAuthenticationArgs;
use cloud_terrastodon_azure_devops::AzureDevOpsAgentPoolEntitlementListForProjectRequest;
use cloud_terrastodon_azure_devops::AzureDevOpsAgentPoolId;
use cloud_terrastodon_azure_devops::AzureDevOpsAgentPoolListRequest;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectId;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectsListRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ConcurrentIteratorExt;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use facet::Facet;
use std::borrow::Cow;
use std::collections::HashMap;
use std::collections::HashSet;
use std::num::NonZeroUsize;
use tracing::info;

/// Print a summary of agent pools and projects that belong to each pool.
#[derive(Facet, Debug, Clone)]
pub struct AzureDevOpsAgentPoolSummaryArgs {
    #[facet(flatten)]
    pub authentication: AzureDevOpsAuthenticationArgs,

    /// Azure DevOps organization name or URL.
    #[facet(figue::named)]
    pub org: AzureDevOpsOrganizationUrl,
}

impl AzureDevOpsAgentPoolSummaryArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        // Fetch pools and projects once
        let auth_selection = self.authentication.resolve(&self.org, executor).await?;
        let pools = AzureDevOpsAgentPoolListRequest {
            org_url: Cow::Borrowed(&self.org),
            auth_selection,
        }
        .execute(executor)
        .await?;
        let projects = AzureDevOpsProjectsListRequest {
            org_url: Cow::Borrowed(&self.org),
            auth_selection,
        }
        .execute(executor)
        .await?;

        // Map project id -> project name and record project ids

        let mut project_ids: Vec<AzureDevOpsProjectId> = Vec::with_capacity(projects.len());
        for p in projects.iter() {
            project_ids.push(p.id);
        }

        // Build pool -> project ids map by fetching entitlements once per project in parallel
        let mut pool_projects: HashMap<AzureDevOpsAgentPoolId, HashSet<AzureDevOpsProjectId>> =
            HashMap::new();

        // Keep ordinary service failures as results until the complete batch
        // drains; cancellation still stops admission and active optional work.
        let entitlements = project_ids
            .iter()
            .map_concurrent(
                executor,
                NonZeroUsize::MIN.saturating_add(7),
                async |child, pid| {
                    let entitlements = AzureDevOpsAgentPoolEntitlementListForProjectRequest {
                        org_url: Cow::Borrowed(&self.org),
                        project: (pid).into(),
                        auth_selection,
                    }
                    .execute(&child)
                    .await;
                    info!(project_id = %pid, "Finished fetching project agent pool entitlements");
                    eyre::Ok(entitlements.map(|entitlements| (pid, entitlements)))
                },
            )
            .await?
            .into_iter()
            .collect::<Result<Vec<_>>>()?;

        for (pid, entitlements) in entitlements {
            for e in entitlements.into_iter() {
                pool_projects.entry(e.pool.id).or_default().insert(*pid);
            }
        }

        Ok(CliOutput::facet_with_text(
            AzureDevOpsAgentPoolSummary {
                org: self.org,
                pools,
                projects,
                pool_projects,
            },
            AzureDevOpsAgentPoolSummary::write_text,
        ))
    }
}
