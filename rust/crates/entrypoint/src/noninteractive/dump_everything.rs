use cloud_terrastodon_core_azure::prelude::fetch_all_resource_groups;
use cloud_terrastodon_core_azure::prelude::fetch_all_subscriptions;
use cloud_terrastodon_core_azure_devops::prelude::fetch_all_azure_devops_projects;
use cloud_terrastodon_core_azure_devops::prelude::fetch_azure_devops_repos_batch;
use cloud_terrastodon_core_pathing::AppDir;
use cloud_terrastodon_core_tofu::prelude::TofuImportBlock;
use cloud_terrastodon_core_tofu::prelude::TofuImporter;
use cloud_terrastodon_core_tofu::prelude::TofuWriter;
use eyre::bail;
use eyre::Context;
use tokio::sync::Semaphore;
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tokio::task::JoinSet;
use tokio::try_join;
use tracing::debug;
use tracing::info;

use crate::interactive::prelude::clean_imports;
use crate::interactive::prelude::clean_processed;

async fn write_import_blocks(
    file_path: impl AsRef<Path>,
    import_blocks: impl IntoIterator<Item = impl Into<TofuImportBlock>>,
) -> eyre::Result<()> {
    let import_blocks: Vec<TofuImportBlock> = import_blocks.into_iter().map(|x| x.into()).collect();
    let len = import_blocks.len();
    TofuWriter::new(&file_path)
        .overwrite(import_blocks)
        .await?
        .format()
        .await?;
    debug!(
        "Wrote {} import blocks to {}",
        len,
        file_path.as_ref().display()
    );
    Ok(())
}

pub async fn dump_everything() -> eyre::Result<()> {
    info!("Clean up previous runs");
    _ = clean_imports().await;
    _ = clean_processed().await;

    let tf_workspace_dirs = write_all_import_blocks().await?;
    import_all(tf_workspace_dirs).await?;

    // info!("Make it pretty");
    // process_generated().await?;

    // info!("Done!");
    // info!(
    //     "The output is available at {}",
    //     AppDir::Processed.as_path_buf().display()
    // );

    // info!("Make sure there is no drift");
    // init_processed().await?;
    // plan_processed().await?;

    return Ok(());
}

async fn write_all_import_blocks() -> eyre::Result<Vec<PathBuf>> {
    info!("Writing all import blocks; fetching a lot of data");
    let (azure_devops_projects, subscriptions, resource_groups) = try_join!(
        fetch_all_azure_devops_projects(),
        fetch_all_subscriptions(),
        fetch_all_resource_groups(),
    )?;

    let mut tf_workspace_dirs: Vec<PathBuf> = Vec::new();

    let azure_devops_project_repos = fetch_azure_devops_repos_batch(
        azure_devops_projects
            .iter()
            .map(|project| project.id.clone())
            .collect(),
    )
    .await?;

    let azure_devops_dir = AppDir::Imports.join("AzureDevOps");

    let mut join_set: JoinSet<eyre::Result<()>> = JoinSet::new();

    for (project, (_project_id, repos)) in azure_devops_projects
        .into_iter()
        .zip(azure_devops_project_repos.into_iter())
    {
        let project_dir = azure_devops_dir.join(project.name.replace(" ", "-"));
        let project_creation_dir = project_dir.join("project_creation");
        let project_repos_dir = project_dir.join("repos");
        let project_tf_file = project_creation_dir.join("project.tf");
        let repos_tf_file = project_repos_dir.join("repos.tf");
        tf_workspace_dirs.push(project_creation_dir);
        tf_workspace_dirs.push(project_repos_dir);
        join_set.spawn(async move {
            try {
                write_import_blocks(project_tf_file, vec![project]).await?;
                write_import_blocks(repos_tf_file, repos).await?;
            }
        });
    }

    let azure_portal_dir = AppDir::Imports.join("AzurePortal");
    let subscriptions_dir = azure_portal_dir.join("subscriptions");

    let subscriptions_by_id = subscriptions
        .into_iter()
        .map(|sub| (sub.id.to_owned(), sub))
        .collect::<HashMap<_, _>>();
    for rg in resource_groups {
        let Some(sub) = subscriptions_by_id.get(&rg.subscription_id) else {
            bail!(
                "Failed to find subscription {} for resource group {}",
                rg.subscription_id,
                rg.name
            );
        };
        let subscription_dir = subscriptions_dir.join(sub.name.replace(" ", "-"));
        let resource_group_dir = subscription_dir.join(rg.name.replace(" ", "-"));

        let boilerplate_file = resource_group_dir.join("boilerplate.tf");
        let resource_group_import_file = resource_group_dir.join("resource-group.tf");

        let azurerm_provider_block = sub.into_provider_block();
        let mut resource_group_import_block: TofuImportBlock = rg.into();
        resource_group_import_block.provider = azurerm_provider_block.as_reference();
        join_set.spawn(async move {
            try {
                TofuWriter::new(&boilerplate_file)
                    .merge([azurerm_provider_block])
                    .await?
                    .format()
                    .await?;

                write_import_blocks(resource_group_import_file, [resource_group_import_block])
                    .await?;
            }
        });
    }

    info!("Waiting for tasks to finish...");
    while let Some(result) = join_set.join_next().await {
        result??;
        info!("{} tasks remaining...", join_set.len());
    }
    info!("All done!");

    // next steps: split by sub and rg
    // azure-portal/subscriptions/mysub/resource-groups/my-resource-group/resource-group-creation
    // azure-portal/subscriptions/mysub/resource-groups/my-resource-group/resource-group-rbac
    // azure-portal/subscriptions/mysub/resource-groups/my-resource-group/resource-group-networking
    // azure-portal/subscriptions/mysub/resource-groups/my-resource-group/resource-group-statefile-storage
    Ok(tf_workspace_dirs)
}

async fn import_all(tf_workspace_dirs: Vec<PathBuf>) -> eyre::Result<()> {
    let tf_workspace_dir_count = tf_workspace_dirs.len();
    info!(
        "Performing import for {} tf workspace dirs",
        tf_workspace_dir_count
    );
    let start = Instant::now();
    let mut join_set: JoinSet<eyre::Result<()>> = JoinSet::new();
    let limit = Arc::new(Semaphore::new(1));
    for dir in tf_workspace_dirs {
        let limit = limit.clone();
        join_set.spawn(async move {
            try {
                let permit = limit.acquire().await?;
                TofuImporter::default().using_dir(&dir).run().await.wrap_err(format!("Importing tf workspace dir \"{}\"", dir.display()))?;
                drop(permit);
            }
        });
    }

    info!("Waiting for tasks to finish...");
    while let Some(result) = join_set.join_next().await {
        result??;
        info!("{} tasks remaining...", join_set.len());
    }

    let end = Instant::now();
    let took = end - start;
    info!(
        "Performed import for {} tf workspace dirs in {}",
        tf_workspace_dir_count,
        humantime::format_duration(took)
    );
    Ok(())
}

#[cfg(test)]
mod test {
    use cloud_terrastodon_core_pathing::Existy;
    use cloud_terrastodon_core_tofu::prelude::TofuImporter;
    use tempfile::Builder;

    #[tokio::test]
    pub async fn terraform_concurrent_init() -> eyre::Result<()> {
        let temp_dir = Builder::new().tempdir()?;
        let num_workspaces = 5;
        for i in 0..num_workspaces {
            let workspace_dir = temp_dir.path().join(format!("workspace_{i:03}"));
            workspace_dir.ensure_dir_exists().await?;
            TofuImporter::new().using_dir(&workspace_dir).run().await?;
        }
        Ok(())
    }
}