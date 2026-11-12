use crate::AzureDevOpsRepositoriesListRequest;
use cloud_terrastodon_auth_types::AzureDevOpsAuthSelection;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsProjectId;
use cloud_terrastodon_azure_devops_types::AzureDevOpsRepo;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ConcurrentIteratorExt;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use eyre::WrapErr;
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;

// Keep organization-wide discovery bounded instead of starting one request
// per project. This is a local scheduling policy, not a service quota.
const MAX_CONCURRENT_PROJECTS: usize = 8;

pub async fn fetch_azure_devops_repos_batch(
    org_url: &AzureDevOpsOrganizationUrl,
    project_ids: Vec<AzureDevOpsProjectId>,
    auth_selection: AzureDevOpsAuthSelection,
    executor: &CloudTerrastodonExecutor,
) -> Result<HashMap<AzureDevOpsProjectId, Arc<Vec<AzureDevOpsRepo>>>> {
    info!("Fetching repos for {} projects", project_ids.len());
    let project_count = project_ids.len();
    let repos: HashMap<AzureDevOpsProjectId, Arc<Vec<AzureDevOpsRepo>>> = project_ids
        .into_iter()
        .map_concurrent(
            executor,
            MAX_CONCURRENT_PROJECTS.try_into()?,
            |job, project_id| async move {
                let repos = AzureDevOpsRepositoriesListRequest {
                    org_url: Cow::Borrowed(org_url),
                    project_id,
                    auth_selection,
                }
                .execute(&job)
                .await
                .wrap_err_with(|| format!("Fetching repos for project {project_id:?}"))?;
                eyre::Ok((project_id, repos))
            },
        )
        .await?
        .into_iter()
        .collect();
    info!(
        "Found {} repos across {} projects",
        repos.values().map(|repos| repos.len()).sum::<usize>(),
        project_count
    );
    Ok(repos)
}
