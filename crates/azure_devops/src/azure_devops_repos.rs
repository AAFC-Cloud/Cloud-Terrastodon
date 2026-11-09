use crate::fetch_all_azure_devops_repos_for_project;
use cloud_terrastodon_auth::AzureDevOpsAuthContext;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsProjectId;
use cloud_terrastodon_azure_devops_types::AzureDevOpsRepo;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ConcurrentIteratorExt;
use eyre::Result;
use eyre::WrapErr;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;

// Keep organization-wide discovery bounded instead of starting one request
// per project. This is a local scheduling policy, not a service quota.
const MAX_CONCURRENT_PROJECTS: usize = 8;

pub async fn fetch_azure_devops_repos_batch(
    org_url: &AzureDevOpsOrganizationUrl,
    project_ids: Vec<AzureDevOpsProjectId>,
    auth_context: &AzureDevOpsAuthContext,
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
                let repos = fetch_all_azure_devops_repos_for_project(
                    org_url,
                    &project_id,
                    auth_context,
                    &job,
                )
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fetch_all_azure_devops_projects;
    use crate::get_default_organization_url;
    use cloud_terrastodon_auth::AuthContext;
    use cloud_terrastodon_auth::AzureDevOpsAuthContext;
    use cloud_terrastodon_command::CancellationToken;

    #[tokio::test]
    async fn test_fetch_all_azure_devops_repos() -> Result<()> {
        CancellationToken::new()
            .with_async_scope(|cancellation| async move {
                let auth_context = AuthContext::explicit_azure_cli();
                let azure_devops_auth_context = AzureDevOpsAuthContext::for_tenant(
                    &auth_context,
                    crate::test_tenant_id::test_tenant_id()?,
                )?;
                let executor =
                    crate::test_executor::test_executor(&azure_devops_auth_context, &cancellation)?;
                let org_url = get_default_organization_url(&cancellation).await?;
                let projects = fetch_all_azure_devops_projects(
                    &org_url,
                    azure_devops_auth_context.selection()?,
                    &executor,
                )
                .await?;
                let mut repo_counts = Vec::new();
                for project in projects.iter().take(5) {
                    let repos = fetch_all_azure_devops_repos_for_project(
                        &org_url,
                        &project.id,
                        &azure_devops_auth_context,
                        &executor,
                    )
                    .await?;
                    assert!(repos.iter().all(|repo| !repo.name.is_empty()));
                    repo_counts.push(repos.len());
                }
                assert!(repo_counts.into_iter().sum::<usize>() > 0);
                Ok(())
            })
            .await
    }
}
