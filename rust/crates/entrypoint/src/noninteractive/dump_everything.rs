use cloud_terrastodon_core_azure::prelude::fetch_all_resource_groups;
use cloud_terrastodon_core_azure::prelude::fetch_all_subscriptions;
use cloud_terrastodon_core_azure_devops::prelude::AzureDevOpsProjectId;
use cloud_terrastodon_core_azure_devops::prelude::fetch_all_azure_devops_projects;
use cloud_terrastodon_core_azure_devops::prelude::fetch_azure_devops_repos_batch;
use cloud_terrastodon_core_azure_devops::prelude::fetch_azure_devops_teams_batch;
use cloud_terrastodon_core_azure_devops::prelude::get_personal_access_token;
use cloud_terrastodon_core_pathing::AppDir;
use cloud_terrastodon_core_pathing::Existy;
use cloud_terrastodon_core_tofu::prelude::FreshTFWorkDir;
use cloud_terrastodon_core_tofu::prelude::GeneratedConfigOutTFWorkDir;
use cloud_terrastodon_core_tofu::prelude::InitializedTFWorkDir;
use cloud_terrastodon_core_tofu::prelude::ProcessedTFWorkDir;
use cloud_terrastodon_core_tofu::prelude::ProviderManager;
use cloud_terrastodon_core_tofu::prelude::TofuBlock;
use cloud_terrastodon_core_tofu::prelude::TofuImportBlock;
use cloud_terrastodon_core_tofu::prelude::TofuWriter;
use cloud_terrastodon_core_tofu::prelude::ValidatedTFWorkDir;
use cloud_terrastodon_core_tofu::prelude::generate_config_out_bulk;
use cloud_terrastodon_core_tofu::prelude::initialize_work_dirs;
use cloud_terrastodon_core_tofu::prelude::reflow_workspace;
use cloud_terrastodon_core_tofu::prelude::validate_work_dirs;
use cloud_terrastodon_core_user_input::prelude::Choice;
use cloud_terrastodon_core_user_input::prelude::FzfArgs;
use cloud_terrastodon_core_user_input::prelude::pick;
use eyre::bail;
use humantime::FormattedDuration;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;
use std::time::Instant;
use strum::VariantArray;
use tokio::fs::read_dir;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::mpsc::unbounded_channel;
use tokio::task::JoinSet;
use tokio::try_join;
use tracing::debug;
use tracing::info;
use tracing::warn;

use crate::interactive::prelude::clean_imports;
use crate::interactive::prelude::clean_processed;

pub async fn measure<F, T>(runnable: F) -> eyre::Result<(T, FormattedDuration)>
where
    F: std::future::Future<Output = eyre::Result<T>>,
    F: Send + 'static,
{
    let start = Instant::now();
    let rtn = runnable.await?;
    let end = Instant::now();
    let took = end - start;
    let took = Duration::from_secs(took.as_secs());
    let duration = humantime::format_duration(took);
    Ok((rtn, duration))
}

pub async fn dump_everything() -> eyre::Result<()> {
    info!("Ensuring Azure DevOps PAT is set for future steps");
    _ = get_personal_access_token().await?;

    #[derive(VariantArray, Debug)]
    enum Behaviour {
        CleanAndWriteImportsAndInitAndValidateAndGenerateAndProcess,
        WriteImportsAndInitAndValidateAndGenerateAndProcess,
        InitAndValidateAndGenerateAndProcess,
        ValidateAndGenerateAndProcess,
        GenerateAndProcess,
        Process,
    }
    let behaviour = pick(FzfArgs {
        choices: Behaviour::VARIANTS
            .iter()
            .map(|behaviour| Choice {
                key: format!("{:?}", behaviour),
                value: behaviour,
            })
            .collect(),
        ..Default::default()
    })?
    .value;

    let (_, took) = measure(async move {
        let should_clean = matches!(
            behaviour,
            Behaviour::CleanAndWriteImportsAndInitAndValidateAndGenerateAndProcess
        );
        if should_clean {
            info!("Clean up previous runs");
            _ = clean_imports().await;
            _ = clean_processed().await;
        }

        let tf_work_dirs = match behaviour {
            Behaviour::CleanAndWriteImportsAndInitAndValidateAndGenerateAndProcess
            | Behaviour::WriteImportsAndInitAndValidateAndGenerateAndProcess => {
                write_all_import_blocks().await?
            }
            _ => discover_existing_dirs().await?,
        };

        let should_init = matches!(
            behaviour,
            Behaviour::CleanAndWriteImportsAndInitAndValidateAndGenerateAndProcess
                | Behaviour::WriteImportsAndInitAndValidateAndGenerateAndProcess
                | Behaviour::InitAndValidateAndGenerateAndProcess
        );
        let tf_work_dirs: Vec<InitializedTFWorkDir> = if should_init {
            let tf_work_dir_count = tf_work_dirs.len();
            info!("Performing init for {} tf work dirs", tf_work_dir_count);
            let (tf_work_dirs, took) =
                measure(async move { initialize_work_dirs(tf_work_dirs).await }).await?;
            info!("Performed init for {tf_work_dir_count} work dirs in {took}");
            tf_work_dirs
        } else {
            tf_work_dirs.into_iter().map(|x| x.into()).collect()
        };

        let should_clean_old_generated_tf_files =
            matches!(behaviour, Behaviour::ValidateAndGenerateAndProcess);
        if should_clean_old_generated_tf_files {
            clean_up_generated_files(&tf_work_dirs).await?;
        }

        let should_validate = matches!(
            behaviour,
            Behaviour::CleanAndWriteImportsAndInitAndValidateAndGenerateAndProcess
                | Behaviour::WriteImportsAndInitAndValidateAndGenerateAndProcess
                | Behaviour::InitAndValidateAndGenerateAndProcess
                | Behaviour::ValidateAndGenerateAndProcess
        );
        let tf_work_dirs: Vec<ValidatedTFWorkDir> = if should_validate {
            let tf_work_dir_count = tf_work_dirs.len();
            let (work_dirs, duration) =
                measure(async move { validate_work_dirs(tf_work_dirs).await }).await?;
            info!("Validated {tf_work_dir_count} work dirs in {duration}");
            work_dirs
        } else {
            tf_work_dirs.into_iter().map(|x| x.into()).collect()
        };

        let should_skip_generated_tf_files = matches!(behaviour, Behaviour::GenerateAndProcess);
        let tf_work_dirs: Vec<ValidatedTFWorkDir> = if should_skip_generated_tf_files {
            prune_dirs_containing_generated_tf_file(tf_work_dirs).await?
        } else {
            tf_work_dirs
        };

        let should_generate = matches!(
            behaviour,
            Behaviour::CleanAndWriteImportsAndInitAndValidateAndGenerateAndProcess
                | Behaviour::WriteImportsAndInitAndValidateAndGenerateAndProcess
                | Behaviour::InitAndValidateAndGenerateAndProcess
                | Behaviour::ValidateAndGenerateAndProcess
                | Behaviour::GenerateAndProcess
        );
        let tf_work_dirs = if should_generate {
            let tf_work_dir_count = tf_work_dirs.len();
            let (tf_work_dirs, duration) =
                measure(async move { generate_config_out_bulk(tf_work_dirs).await }).await?;
            info!("Generated configs for {tf_work_dir_count} work dirs in {duration}");
            tf_work_dirs
        } else {
            tf_work_dirs.into_iter().map(|x| x.into()).collect()
        };

        let (tf_work_dirs, duration) =
            measure(async move { process_generated_many(tf_work_dirs).await }).await?;
        info!(
            "Processed generated terraform code in {} dirs in {duration}",
            tf_work_dirs.len()
        );
        // info!("Make sure there is no drift");
        // init_processed().await?;
        // plan_processed().await?;
        Ok(())
    })
    .await?;

    info!("Overall dump took {took}");
    return Ok(());
}

async fn process_generated_many(
    tf_work_dirs: Vec<GeneratedConfigOutTFWorkDir>,
) -> eyre::Result<Vec<ProcessedTFWorkDir>> {
    let mut rtn: Vec<ProcessedTFWorkDir> = Default::default();
    let out_dir: PathBuf = AppDir::Processed.into();
    _ = tokio::fs::remove_dir_all(&out_dir).await;
    let imports_dir = AppDir::Imports.as_path_buf().canonicalize()?;
    let mut write_jobs: JoinSet<eyre::Result<()>> = JoinSet::new();
    for work_dir in tf_work_dirs {
        let out_dir = AppDir::Processed.join(
            work_dir
                .canonicalize()?
                .strip_prefix(&imports_dir)
                .map(PathBuf::from)?,
        );
        let files = reflow_workspace(&work_dir, &out_dir).await?;
        rtn.push(ProcessedTFWorkDir::new(out_dir));
        for (path, content) in files {
            write_jobs.spawn(async move {
                path.ensure_parent_dir_exists().await?;
                TofuWriter::new(path)
                    .format_on_write()
                    .overwrite(content)
                    .await?;
                Ok(())
            });
        }
    }
    while let Some(x) = write_jobs.join_next().await {
        x??;
        info!("Writing files, {} tasks remain...", write_jobs.len());
    }

    Ok(rtn)
}

async fn prune_dirs_containing_generated_tf_file(
    tf_work_dirs: Vec<ValidatedTFWorkDir>,
) -> eyre::Result<Vec<ValidatedTFWorkDir>> {
    let mut rtn: Vec<ValidatedTFWorkDir> = Vec::new();
    debug!(
        "Pruning dirs already containing generated.tf file, started with {} dirs",
        tf_work_dirs.len()
    );
    let mut join_set: JoinSet<eyre::Result<Option<ValidatedTFWorkDir>>> = JoinSet::new();
    for work_dir in tf_work_dirs {
        let generated_path = work_dir.join("generated.tf");
        join_set.spawn(async move {
            let exists = matches!(tokio::fs::try_exists(&generated_path).await, Ok(true));
            Ok(if exists { None } else { Some(work_dir) })
        });
    }
    while let Some(x) = join_set.join_next().await {
        info!(
            "Checking if dir already contains generated.tf files, {} tasks remain",
            join_set.len()
        );
        if let Some(work_dir) = x?? {
            rtn.push(work_dir);
        }
    }
    Ok(rtn)
}

async fn clean_up_generated_files(tf_work_dirs: &[InitializedTFWorkDir]) -> eyre::Result<()> {
    debug!("Checking for generated.tf files from previous runs, going to delete them");
    let mut join_set: JoinSet<eyre::Result<()>> = JoinSet::new();
    for work_dir in tf_work_dirs {
        let generated_path = work_dir.join("generated.tf");
        join_set.spawn(async move {
            match tokio::fs::try_exists(&generated_path).await {
                Ok(true) => {
                    warn!("Removing old generated file: {}", generated_path.display());
                    tokio::fs::remove_file(generated_path).await?;
                }
                _ => {}
            }
            Ok(())
        });
    }
    while let Some(x) = join_set.join_next().await {
        info!(
            "Cleaning up old generated.tf files, {} tasks remain",
            join_set.len()
        );
        x??;
    }
    Ok(())
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
        .format_on_write()
        .overwrite(import_blocks)
        .await?;
    debug!(
        "Wrote {} import blocks to {}",
        len,
        file_path.as_ref().display()
    );
    Ok(())
}

async fn discover_existing_dirs() -> eyre::Result<Vec<FreshTFWorkDir>> {
    let azure_devops_dir: PathBuf = AppDir::Imports.join("AzureDevOps");
    let azure_portal_dir: PathBuf = AppDir::Imports.join("AzurePortal");
    let mut rtn = Vec::new();

    if azure_devops_dir.exists() {
        let mut project_dirs = read_dir(azure_devops_dir).await?;
        while let Some(project_dir_entry) = project_dirs.next_entry().await? {
            let kind = project_dir_entry.file_type().await?;
            if !kind.is_dir() {
                warn!(
                    "Unexpected non-directory item found: {}",
                    project_dir_entry.path().display()
                );
                continue;
            }
            let mut project_children = read_dir(project_dir_entry.path()).await?;
            while let Some(aspect_entry) = project_children.next_entry().await? {
                let kind = aspect_entry.file_type().await?;
                if !kind.is_dir() {
                    warn!(
                        "Unexpected non-directory item found: {}",
                        project_dir_entry.path().display()
                    );
                    continue;
                }
                rtn.push(aspect_entry.path());
            }
        }
    }
    if azure_portal_dir.exists() {
        let subscriptions_dir = azure_portal_dir.join("subscriptions");
        if subscriptions_dir.exists() {
            let mut subscription_dirs = read_dir(subscriptions_dir).await?;
            while let Some(subscription_entry) = subscription_dirs.next_entry().await? {
                let kind = subscription_entry.file_type().await?;
                if !kind.is_dir() {
                    warn!(
                        "Unexpected non-directory item found: {}",
                        subscription_entry.path().display()
                    );
                    continue;
                }
                let mut resource_groups = read_dir(subscription_entry.path()).await?;
                while let Some(resource_group_entry) = resource_groups.next_entry().await? {
                    let kind = resource_group_entry.file_type().await?;
                    if !kind.is_dir() {
                        warn!(
                            "Unexpected non-directory item found: {}",
                            resource_group_entry.path().display()
                        );
                        continue;
                    }
                    rtn.push(resource_group_entry.path());
                }
            }
        }
    }

    let rtn = rtn.into_iter().map(|x| x.into()).collect();
    Ok(rtn)
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

    info!("Writing Azure DevOps tf files");
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
        tf_work_dirs.push(resource_group_dir.clone());

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
                    .format_on_write()
                    .merge([azurerm_provider_block])
                    .await?;

                write_import_blocks(
                    resource_group_import_file,
                    [resource_group_import_block],
                    sender,
                )
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
        info!("{} file write tasks remaining...", join_set.len());
    }
    all_in_one_imports_rx.close();

    info!("Writing the all-in-one");
    let all_in_one_file = all_in_one_dir.join("all-in-one.tf");
    let mut import_blocks: Vec<TofuImportBlock> = vec![];
    while let Some(import_block) = all_in_one_imports_rx.recv().await {
        import_blocks.extend(import_block);
    }
    let mut together: Vec<TofuBlock> = Vec::new();
    for block in provider_blocks {
        together.push(block.into());
    }
    for block in import_blocks {
        together.push(block.into());
    }

    TofuWriter::new(&all_in_one_file)
        .format_on_write()
        .merge(together)
        .await?;

    info!("All done!");

    Ok(tf_work_dirs
        .into_iter()
        .map(|x| FreshTFWorkDir::from(x))
        .collect())
}
