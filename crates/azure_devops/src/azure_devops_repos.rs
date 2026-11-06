use crate::fetch_all_azure_devops_repos_for_project;
use cloud_terrastodon_auth::AzureDevOpsAuthContext;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsProjectId;
use cloud_terrastodon_azure_devops_types::AzureDevOpsRepo;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use eyre::WrapErr;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::task::JoinSet;
use tracing::info;

pub async fn fetch_azure_devops_repos_batch(
    org_url: &AzureDevOpsOrganizationUrl,
    project_ids: Vec<AzureDevOpsProjectId>,
    auth_context: &AzureDevOpsAuthContext,
    executor: &CloudTerrastodonExecutor,
) -> Result<HashMap<AzureDevOpsProjectId, Arc<Vec<AzureDevOpsRepo>>>> {
    executor
        .cancellable_scope(|batch| async move {
            info!("Fetching repos for {} projects", project_ids.len());
            let mut rtn: HashMap<AzureDevOpsProjectId, Arc<Vec<AzureDevOpsRepo>>> = HashMap::new();
            let mut set = JoinSet::new();
            let project_count = project_ids.len();
            for project_id in project_ids {
                batch.cancellation().bail_if_cancelled()?;
                let org_url = org_url.clone();
                let auth_context = auth_context.clone();
                set.spawn(batch.cancellable_scope(move |job| async move {
                    let repos = fetch_all_azure_devops_repos_for_project(
                        &org_url,
                        &project_id,
                        &auth_context,
                        &job,
                    )
                    .await
                    .wrap_err_with(|| format!("Fetching repos for project {project_id:?}"))?;
                    Ok::<_, eyre::Report>((project_id, repos))
                }));
            }
            // These jobs only list repositories, so an error can abandon the remaining reads.
            while let Some(res) = set.join_next().await {
                let (project_id, repos) = res??;
                rtn.insert(project_id, repos);
            }
            info!(
                "Found {} repos across {} projects",
                rtn.values().map(|x| x.len()).sum::<usize>(),
                project_count
            );
            Ok(rtn)
        })
        .await
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
                    &azure_devops_auth_context,
                    &executor,
                )
                .await?;
                let mut repo_counts = Vec::new();
                for project in projects.into_iter().take(5) {
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
