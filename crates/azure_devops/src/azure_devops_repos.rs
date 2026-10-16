use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsProjectId;
use cloud_terrastodon_azure_devops_types::AzureDevOpsRepo;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use eyre::Result;
use eyre::WrapErr;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::task::JoinSet;
use tracing::debug;
use tracing::info;

pub async fn fetch_all_azure_devops_repos_for_project(
    org_url: &AzureDevOpsOrganizationUrl,
    project_id: &AzureDevOpsProjectId,
    cancellation: &CancellationScope,
) -> Result<Vec<AzureDevOpsRepo>> {
    debug!("Fetching repos for project {project_id:?}");
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.cancellation_safe();
    cmd.args([
        "repos",
        "list",
        "--organization",
        org_url.to_string().as_ref(),
        "--output",
        "json",
        "--project",
        project_id.to_string().as_ref(),
    ]);
    cmd.cache(CacheKey::new(PathBuf::from_iter([
        "az",
        "repos",
        "list",
        project_id.to_string().as_ref(),
    ])));
    let repos: Vec<AzureDevOpsRepo> = cmd.run(&cancellation).await?;
    debug!("Found {} repos for {project_id:?}", repos.len());
    Ok(repos)
}

pub async fn fetch_azure_devops_repos_batch(
    org_url: &AzureDevOpsOrganizationUrl,
    project_ids: Vec<AzureDevOpsProjectId>,
    cancellation: &CancellationScope,
) -> Result<HashMap<AzureDevOpsProjectId, Vec<AzureDevOpsRepo>>> {
    cancellation
        .cancellable_scope(|batch| async move {
            info!("Fetching repos for {} projects", project_ids.len());
            let mut rtn: HashMap<AzureDevOpsProjectId, Vec<AzureDevOpsRepo>> = HashMap::new();
            let mut set = JoinSet::new();
            let project_count = project_ids.len();
            for project_id in project_ids {
                batch.bail_if_cancelled()?;
                let org_url = org_url.clone();
                set.spawn(batch.cancellable_scope(move |job| async move {
                    let repos =
                        fetch_all_azure_devops_repos_for_project(&org_url, &project_id, &job)
                            .await
                            .wrap_err_with(|| {
                                format!("Fetching repos for project {project_id:?}")
                            })?;
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
    use cloud_terrastodon_command::CancellationToken;
    use cloud_terrastodon_credentials::AuthContext;
    use cloud_terrastodon_credentials::AzureDevOpsAuthContext;

    #[tokio::test]
    async fn test_fetch_all_azure_devops_repos() -> Result<()> {
        CancellationToken::new()
            .with_async_scope(|cancellation| async move {
                let org_url = get_default_organization_url(&cancellation).await?;
                let auth_context = AuthContext::explicit_azure_cli();
                let azure_devops_auth_context = AzureDevOpsAuthContext::new(&auth_context)?;
                let projects = fetch_all_azure_devops_projects(
                    &org_url,
                    &azure_devops_auth_context,
                    &cancellation,
                )
                .await?;
                let mut repo_counts = Vec::new();
                for project in projects.into_iter().take(5) {
                    let repos = fetch_all_azure_devops_repos_for_project(
                        &org_url,
                        &project.id,
                        &cancellation,
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
