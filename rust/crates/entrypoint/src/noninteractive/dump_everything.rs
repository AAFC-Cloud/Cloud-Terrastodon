use cloud_terrastodon_core_azure::prelude::fetch_all_resource_groups;
use cloud_terrastodon_core_azure::prelude::fetch_all_subscriptions;
use cloud_terrastodon_core_azure_devops::prelude::get_personal_access_token;
use cloud_terrastodon_core_azure_devops::prelude::AzureDevOpsProjectId;
use cloud_terrastodon_core_azure_devops::prelude::fetch_all_azure_devops_projects;
use cloud_terrastodon_core_azure_devops::prelude::fetch_azure_devops_repos_batch;
use cloud_terrastodon_core_azure_devops::prelude::fetch_azure_devops_teams_batch;
use cloud_terrastodon_core_pathing::AppDir;
use cloud_terrastodon_core_pathing::Existy;
use cloud_terrastodon_core_tofu::prelude::FreshTFWorkDir;
use cloud_terrastodon_core_tofu::prelude::ProviderManager;
use cloud_terrastodon_core_tofu::prelude::TofuImportBlock;
use cloud_terrastodon_core_tofu::prelude::TofuWriter;
use cloud_terrastodon_core_tofu::prelude::generate_config_out_bulk;
use cloud_terrastodon_core_tofu::prelude::initialize_work_dirs;
use cloud_terrastodon_core_tofu::prelude::validate_work_dirs;
use eyre::bail;
use tokio::sync::mpsc::UnboundedSender;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;
use std::time::Instant;
use tokio::sync::mpsc::unbounded_channel;
use tokio::task::JoinSet;
use tokio::try_join;
use tracing::debug;
use tracing::info;
use tracing::warn;

use crate::interactive::prelude::clean_imports;
use crate::interactive::prelude::clean_processed;

pub async fn dump_everything() -> eyre::Result<()> {
    info!("Ensuring Azure DevOps PAT is set for future steps");
    _ = get_personal_access_token().await?;

    info!("Clean up previous runs");
    _ = clean_imports().await;
    _ = clean_processed().await;

    let tf_work_dirs = write_all_import_blocks().await?;
    import_all(tf_work_dirs).await?;

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

async fn write_import_blocks(
    file_path: impl AsRef<Path>,
    import_blocks: impl IntoIterator<Item = impl Into<TofuImportBlock>>,
    all_in_one: UnboundedSender<Vec<TofuImportBlock>>,
) -> eyre::Result<()> {
    let import_blocks: Vec<TofuImportBlock> = import_blocks.into_iter().map(|x| x.into()).collect();
    all_in_one.send(import_blocks.clone())?;
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

async fn write_all_import_blocks() -> eyre::Result<Vec<FreshTFWorkDir>> {
    info!("Writing all import blocks; fetching a lot of data");
    let (azure_devops_projects, subscriptions, resource_groups) = try_join!(
        fetch_all_azure_devops_projects(),
        fetch_all_subscriptions(),
        fetch_all_resource_groups(),
    )?;

    let mut tf_work_dirs: Vec<PathBuf> = Vec::new();
    let (all_in_one_imports_tx, mut all_in_one_imports_rx) =
        unbounded_channel::<Vec<TofuImportBlock>>();

    let project_ids: Vec<AzureDevOpsProjectId> = azure_devops_projects
        .iter()
        .map(|project| project.id.clone())
        .collect();
    let mut azure_devops_project_repos =
        fetch_azure_devops_repos_batch(project_ids.clone()).await?;

    let mut azure_devops_project_teams = fetch_azure_devops_teams_batch(project_ids).await?;

    let azure_devops_dir = AppDir::Imports.join("AzureDevOps");

    let mut join_set: JoinSet<eyre::Result<()>> = JoinSet::new();

    info!("Writing Azure Devops tf files");
    for project in azure_devops_projects {
        let Some(repos) = azure_devops_project_repos.remove(&project.id) else {
            warn!("Failed to get repos for project {project:?}");
            continue;
        };
        let Some(teams) = azure_devops_project_teams.remove(&project.id) else {
            warn!("Failed to get teams for project {project:?}");
            continue;
        };

        let project_dir = azure_devops_dir.join(project.name.replace(" ", "-"));

        let project_creation_dir = project_dir.join("project_creation");
        project_creation_dir.ensure_dir_exists().await?;

        let project_repos_dir = project_dir.join("repos");
        project_repos_dir.ensure_dir_exists().await?;

        let project_teams_dir = project_dir.join("teams");
        project_teams_dir.ensure_dir_exists().await?;

        let project_tf_file = project_creation_dir.join("project.tf");
        let repos_tf_file = project_repos_dir.join("repos.tf");
        let teams_tf_file = project_teams_dir.join("teams.tf");

        tf_work_dirs.push(project_creation_dir.clone());
        tf_work_dirs.push(project_repos_dir.clone());
        tf_work_dirs.push(project_teams_dir.clone());

        join_set.spawn(async move {
            try {
                let provider_manager = ProviderManager::try_new()?;
                provider_manager
                    .write_default_provider_configs(&project_creation_dir)
                    .await?;
            }
        });
        join_set.spawn(async move {
            try {
                let provider_manager = ProviderManager::try_new()?;

                provider_manager
                    .write_default_provider_configs(&project_repos_dir)
                    .await?;
            }
        });
        join_set.spawn(async move {
            try {
                let provider_manager = ProviderManager::try_new()?;
                provider_manager
                    .write_default_provider_configs(&project_teams_dir)
                    .await?;
            }
        });

        let sender = all_in_one_imports_tx.clone();
        join_set.spawn(async move {
            try {
                write_import_blocks(project_tf_file, vec![project], sender).await?;
            }
        });

        let sender = all_in_one_imports_tx.clone();
        join_set.spawn(async move {
            try {
                write_import_blocks(repos_tf_file, repos, sender).await?;
            }
        });

        let sender = all_in_one_imports_tx.clone();
        join_set.spawn(async move {
            try {
                write_import_blocks(teams_tf_file, teams, sender).await?;
            }
        });
    }

    info!("Writing Azure Portal tf files");
    let azure_portal_dir = AppDir::Imports.join("AzurePortal");
    let subscriptions_dir = azure_portal_dir.join("subscriptions");

    let subscriptions_by_id = subscriptions
        .into_iter()
        .map(|sub| (sub.id.to_owned(), sub))
        .collect::<HashMap<_, _>>();
    let mut provider_blocks: HashSet<_> = Default::default();
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
        provider_blocks.insert(azurerm_provider_block.clone());
        let mut resource_group_import_block: TofuImportBlock = rg.into();
        resource_group_import_block.provider = azurerm_provider_block.as_reference();
        
        let sender = all_in_one_imports_tx.clone();
        join_set.spawn(async move {
            try {
                TofuWriter::new(&boilerplate_file)
                    .merge([azurerm_provider_block])
                    .await?
                    .format()
                    .await?;

                write_import_blocks(resource_group_import_file, [resource_group_import_block], sender)
                    .await?;
            }
        });
    }

    info!("Prepping all-in-one dir");
    let all_in_one_dir = AppDir::Imports.join("all_in_one");
    tf_work_dirs.push(all_in_one_dir.clone());

    {
        let all_in_one_dir = all_in_one_dir.clone();
        join_set.spawn(async move {
            try {
                let provider_manager = ProviderManager::try_new()?;
                provider_manager
                    .write_default_provider_configs(&all_in_one_dir)
                    .await?;
            }
        });
    }


    info!("Waiting for tasks to finish...");
    while let Some(result) = join_set.join_next().await {
        result??;
        info!("{} tasks remaining...", join_set.len());
    }
    all_in_one_imports_rx.close();

    info!("Writing the all-in-one");
    let all_in_one_file = all_in_one_dir.join("all-in-one.tf");
    let mut import_blocks: Vec<TofuImportBlock> = vec![];
    while let Some(import_block) = all_in_one_imports_rx.recv().await {
        import_blocks.extend(import_block);
    }
    TofuWriter::new(&all_in_one_file)
        .merge(provider_blocks)
        .await?
        .merge(import_blocks)
        .await?
        .format()
        .await?;

    info!("All done!");

    Ok(tf_work_dirs
        .into_iter()
        .map(|x| FreshTFWorkDir::from(x))
        .collect())
}

async fn import_all(work_dirs: Vec<FreshTFWorkDir>) -> eyre::Result<()> {
    let tf_work_dir_count = work_dirs.len();
    info!("Performing import for {} tf work dirs", tf_work_dir_count);
    let start = Instant::now();

    let work_dirs = initialize_work_dirs(work_dirs).await?;
    let work_dirs = validate_work_dirs(work_dirs).await?;
    generate_config_out_bulk(work_dirs).await?;

    let end = Instant::now();
    let took = end - start;
    info!(
        "Performed import for {} tf work dirs in {}",
        tf_work_dir_count,
        humantime::format_duration(took)
    );
    Ok(())
}
