use cloud_terrastodon_azure::prelude::fetch_all_resource_groups;
use cloud_terrastodon_azure::prelude::fetch_all_subscriptions;
use cloud_terrastodon_azure_devops::prelude::AzureDevOpsProjectId;
use cloud_terrastodon_azure_devops::prelude::fetch_all_azure_devops_projects;
use cloud_terrastodon_azure_devops::prelude::fetch_azure_devops_repos_batch;
use cloud_terrastodon_azure_devops::prelude::fetch_azure_devops_teams_for_project;
use cloud_terrastodon_azure_devops::prelude::get_default_organization_url;
use cloud_terrastodon_azure_devops::prelude::get_personal_access_token;
use cloud_terrastodon_command::ParallelFallibleWorkQueue;
use cloud_terrastodon_hcl::discovery::DiscoveryDepth;
use cloud_terrastodon_hcl::discovery::discover_hcl;
use cloud_terrastodon_hcl::prelude::FreshTFWorkDir;
use cloud_terrastodon_hcl::prelude::GeneratedConfigOutTFWorkDir;
use cloud_terrastodon_hcl::prelude::HclBlock;
use cloud_terrastodon_hcl::prelude::HclImportBlock;
use cloud_terrastodon_hcl::prelude::HclWriter;
use cloud_terrastodon_hcl::prelude::InitializedTFWorkDir;
use cloud_terrastodon_hcl::prelude::ProcessedTFWorkDir;
use cloud_terrastodon_hcl::prelude::ProviderManager;
use cloud_terrastodon_hcl::prelude::ValidatedTFWorkDir;
use cloud_terrastodon_hcl::prelude::edit::structure::Body;
use cloud_terrastodon_hcl::prelude::generate_config_out_bulk;
use cloud_terrastodon_hcl::prelude::initialize_work_dirs;
use cloud_terrastodon_hcl::prelude::validate_work_dirs;
use cloud_terrastodon_hcl::reflow::reflow_hcl;
use cloud_terrastodon_pathing::AppDir;
use cloud_terrastodon_pathing::Existy;
use cloud_terrastodon_user_input::PickerTui;
use cloud_terrastodon_zombies::prompt_kill_processes_using_dirs;
use eyre::bail;
use humantime::FormattedDuration;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;
use strum::VariantArray;
use tokio::fs::read_dir;
use tokio::sync::Semaphore;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::mpsc::unbounded_channel;
use tokio::task::JoinSet;
use tokio::try_join;
use tracing::debug;
use tracing::info;
use tracing::warn;

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
    let (rtn, took) = measure(dump_everything_inner()).await?;
    info!("Overall dump took {took}");
    Ok(rtn)
}
#[derive(VariantArray, Debug, Clone, Copy)]
enum Strategy {
    AllInOne,
    Split,
    Both,
}
impl std::fmt::Display for Strategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Strategy::AllInOne => write!(f, "All-in-one tf file"),
            Strategy::Split => write!(f, "Split tf files by resource"),
            Strategy::Both => write!(f, "Both all-in-one and split"),
        }
    }
}
impl Strategy {
    fn split(&self) -> bool {
        match self {
            Strategy::AllInOne => false,
            Strategy::Split => true,
            Strategy::Both => true,
        }
    }
    fn all_in_one(&self) -> bool {
        match self {
            Strategy::AllInOne => true,
            Strategy::Split => false,
            Strategy::Both => true,
        }
    }
}
pub async fn dump_everything_inner() -> eyre::Result<()> {
    let strategy = *PickerTui::new()
        .set_header("Dump strategy")
        .pick_one(Strategy::VARIANTS)?;

    #[derive(VariantArray, Debug)]
    enum Behaviour {
        CleanAndWriteImportsAndInitAndValidateAndGenerateAndProcess,
        WriteImportsAndInitAndValidateAndGenerateAndProcess,
        InitAndValidateAndGenerateAndProcess,
        ValidateAndGenerateAndProcess,
        GenerateAndProcess,
        Process,
    }
    impl std::fmt::Display for Behaviour {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Behaviour::CleanAndWriteImportsAndInitAndValidateAndGenerateAndProcess => {
                    write!(
                        f,
                        "Clean + Write Imports + Init + Validate + Generate + Process"
                    )
                }
                Behaviour::WriteImportsAndInitAndValidateAndGenerateAndProcess => {
                    write!(f, "Write Imports + Init + Validate + Generate + Process")
                }
                Behaviour::InitAndValidateAndGenerateAndProcess => {
                    write!(f, "Init + Validate + Generate + Process")
                }
                Behaviour::ValidateAndGenerateAndProcess => {
                    write!(f, "Validate + Generate + Process")
                }
                Behaviour::GenerateAndProcess => {
                    write!(f, "Generate + Process")
                }
                Behaviour::Process => {
                    write!(f, "Process")
                }
            }
        }
    }
    let behaviour = PickerTui::new()
        .set_header("What do you want to run?")
        .pick_one(Behaviour::VARIANTS)?;

    let should_clean = matches!(
        behaviour,
        Behaviour::CleanAndWriteImportsAndInitAndValidateAndGenerateAndProcess
    );
    if should_clean {
        info!("Clean up previous runs");
        prompt_kill_processes_using_dirs(
            [AppDir::Imports, AppDir::Processed]
                .into_iter()
                .map(|x| x.as_path_buf())
                .filter(|x| x.exists()),
                "Previous runs can leave dangling processes if aborted. Select the ones you want to kill".to_string()
        )?;
        for dir in [AppDir::Imports, AppDir::Processed] {
            if dir.exists_async().await? {
                tokio::fs::remove_dir_all(dir.as_path_buf()).await?;
            }
        }
    }

    let tf_work_dirs = match behaviour {
        Behaviour::CleanAndWriteImportsAndInitAndValidateAndGenerateAndProcess
        | Behaviour::WriteImportsAndInitAndValidateAndGenerateAndProcess => {
            write_all_import_blocks(strategy).await?
        }
        _ => discover_existing_dirs(strategy).await?,
    };
    if matches!(strategy, Strategy::AllInOne) {
        assert_eq!(tf_work_dirs.len(), 1);
    }

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
}

async fn process_generated_many(
    tf_work_dirs: Vec<GeneratedConfigOutTFWorkDir>,
) -> eyre::Result<Vec<ProcessedTFWorkDir>> {
    let mut rtn: Vec<ProcessedTFWorkDir> = Default::default();
    let out_dir: PathBuf = AppDir::Processed.into();
    if out_dir.exists_async().await? {
        tokio::fs::remove_dir_all(&out_dir).await?;
    }
    let imports_dir = AppDir::Imports.as_path_buf().canonicalize()?;
    struct WorkOutcome {
        out_dir: PathBuf,
        file_contents: HashMap<PathBuf, Body>,
    }
    let mut reflow_jobs: JoinSet<eyre::Result<WorkOutcome>> = JoinSet::new();
    let rate_limit = Arc::new(Semaphore::new(16));
    for work_dir in tf_work_dirs {
        let imports_dir_clone = imports_dir.clone();
        let rate_limit = rate_limit.clone();
        reflow_jobs.spawn(async move {
            let permit = rate_limit.acquire().await?;
            let out_dir = AppDir::Processed.join(
                work_dir
                    .canonicalize()?
                    .strip_prefix(&imports_dir_clone)
                    .map(PathBuf::from)?,
            );

            let hcl = discover_hcl(work_dir, DiscoveryDepth::Shallow).await?;
            let hcl = reflow_hcl(hcl).await?;
            drop(permit);
            Ok(WorkOutcome {
                out_dir: out_dir.clone(),
                file_contents: hcl,
            })
        });
    }

    let mut write_jobs: JoinSet<eyre::Result<()>> = JoinSet::new();
    let rate_limit = Arc::new(Semaphore::new(16));
    while let Some(reflow_result) = reflow_jobs.join_next().await {
        let WorkOutcome {
            out_dir,
            file_contents,
        } = reflow_result??;

        info!(
            "Reflowing workspace and spawning write tasks, {} remain",
            reflow_jobs.len()
        );

        rtn.push(ProcessedTFWorkDir::new(out_dir.clone()));

        for (path, content) in file_contents {
            let path_clone = path.clone();
            let rate_limit = rate_limit.clone();
            write_jobs.spawn(async move {
                let permit = rate_limit.acquire().await?;
                path_clone.ensure_parent_dir_exists().await?;
                HclWriter::new(path_clone)
                    .format_on_write()
                    .overwrite(content)
                    .await?;
                drop(permit);
                Ok(())
            });
        }

        // we don't need to write provider configs for processed dir since it will get the terraform config from the import dir
        // write_jobs.spawn(async move {
        //     let provider_manager = ProviderManager::try_new()?;
        //     provider_manager
        //         .write_default_provider_configs(&out_dir)
        //         .await?;
        //     Ok(())
        // });
    }
    while let Some(write_result) = write_jobs.join_next().await {
        write_result??;
        info!(
            "Writing processed files, {} tasks remain...",
            write_jobs.len()
        );
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
            if let Ok(true) = tokio::fs::try_exists(&generated_path).await {
                warn!("Removing old generated file: {}", generated_path.display());
                tokio::fs::remove_file(generated_path).await?;
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
    import_blocks: impl IntoIterator<Item = impl Into<HclImportBlock>>,
    all_in_one: UnboundedSender<Vec<HclImportBlock>>,
    strategy: Strategy,
) -> eyre::Result<()> {
    let import_blocks: Vec<HclImportBlock> = import_blocks.into_iter().map(|x| x.into()).collect();
    if strategy.all_in_one() {
        all_in_one.send(import_blocks.clone())?;
    }
    if strategy.split() {
        let len = import_blocks.len();
        HclWriter::new(&file_path)
            .format_on_write()
            .overwrite(import_blocks)
            .await?;
        debug!(
            "Wrote {} import blocks to {}",
            len,
            file_path.as_ref().display()
        );
    }
    Ok(())
}

async fn discover_existing_dirs(strategy: Strategy) -> eyre::Result<Vec<FreshTFWorkDir>> {
    let all_in_one_dir: PathBuf = AppDir::Imports.join("all_in_one");
    let mut rtn = Vec::new();

    if strategy.all_in_one() && all_in_one_dir.exists() {
        rtn.push(all_in_one_dir);
    }
    if strategy.split() {
        let azure_devops_dir: PathBuf = AppDir::Imports.join("AzureDevOps");
        let azure_portal_dir: PathBuf = AppDir::Imports.join("AzurePortal");
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
    }

    let rtn = rtn.into_iter().map(|x| x.into()).collect();
    Ok(rtn)
}

async fn write_all_import_blocks(strategy: Strategy) -> eyre::Result<Vec<FreshTFWorkDir>> {
    info!("Writing all import blocks; fetching a lot of data");
    let org_url = get_default_organization_url().await?;
    let (azure_devops_projects, subscriptions, resource_groups) = try_join!(
        fetch_all_azure_devops_projects(&org_url),
        fetch_all_subscriptions(),
        fetch_all_resource_groups(),
    )?;

    let mut tf_work_dirs: Vec<PathBuf> = Vec::new();
    let (all_in_one_imports_tx, mut all_in_one_imports_rx) =
        unbounded_channel::<Vec<HclImportBlock>>();

    let project_ids: Vec<AzureDevOpsProjectId> = azure_devops_projects
        .iter()
        .map(|project| project.id.clone())
        .collect();
    let mut azure_devops_project_repos =
        fetch_azure_devops_repos_batch(&org_url, project_ids.clone()).await?;

    let mut azure_devops_project_teams = {
        async move {
            let mut work = ParallelFallibleWorkQueue::new("Fetching Azure DevOps teams", 10);
            for project_id in project_ids {
                let org_url = org_url.clone();
                work.enqueue(async move {
                    let teams = fetch_azure_devops_teams_for_project(&org_url, &project_id);
                    teams.await.map(|teams| (project_id.clone(), teams))
                });
            }
            let results = work.join().await?.into_iter().collect::<HashMap<_, _>>();
            eyre::Ok(results)
        }
    }
    .await?;

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
        let project_repos_dir = project_dir.join("repos");
        let project_teams_dir = project_dir.join("teams");

        let project_tf_file = project_creation_dir.join("project.tf");
        let repos_tf_file = project_repos_dir.join("repos.tf");
        let teams_tf_file = project_teams_dir.join("teams.tf");

        if strategy.split() {
            project_creation_dir.ensure_dir_exists().await?;
            project_repos_dir.ensure_dir_exists().await?;
            project_teams_dir.ensure_dir_exists().await?;

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
        }

        let sender = all_in_one_imports_tx.clone();
        join_set.spawn(async move {
            try {
                write_import_blocks(project_tf_file, vec![project], sender, strategy).await?;
            }
        });

        let sender = all_in_one_imports_tx.clone();
        join_set.spawn(async move {
            try {
                write_import_blocks(repos_tf_file, repos, sender, strategy).await?;
            }
        });

        let sender = all_in_one_imports_tx.clone();
        join_set.spawn(async move {
            try {
                write_import_blocks(teams_tf_file, teams, sender, strategy).await?;
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
        if strategy.split() {
            tf_work_dirs.push(resource_group_dir.clone());
        }

        let boilerplate_file = resource_group_dir.join("boilerplate.tf");
        let resource_group_import_file = resource_group_dir.join("resource-group.tf");

        let azurerm_provider_block = sub.into_provider_block();
        provider_blocks.insert(azurerm_provider_block.clone());
        let mut resource_group_import_block: HclImportBlock = rg.into();
        resource_group_import_block.provider = azurerm_provider_block.as_reference();

        let sender = all_in_one_imports_tx.clone();
        join_set.spawn(async move {
            try {
                if strategy.split() {
                    HclWriter::new(&boilerplate_file)
                        .format_on_write()
                        .merge([azurerm_provider_block])
                        .await?;
                }

                write_import_blocks(
                    resource_group_import_file,
                    [resource_group_import_block],
                    sender,
                    strategy,
                )
                .await?;
            }
        });
    }

    info!("Prepping all-in-one dir");
    let all_in_one_dir = AppDir::Imports.join("all_in_one");
    if strategy.all_in_one() {
        tf_work_dirs.push(all_in_one_dir.clone());
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
        info!("{} import write tasks remaining...", join_set.len());
    }
    all_in_one_imports_rx.close();

    info!("Writing the all-in-one");
    if strategy.all_in_one() {
        let all_in_one_file = all_in_one_dir.join("all-in-one.tf");
        let mut import_blocks: Vec<HclImportBlock> = vec![];
        while let Some(import_block) = all_in_one_imports_rx.recv().await {
            import_blocks.extend(import_block);
        }
        let mut together: Vec<HclBlock> = Vec::new();
        for block in provider_blocks {
            together.push(block.into());
        }
        for block in import_blocks {
            together.push(block.into());
        }

        HclWriter::new(&all_in_one_file)
            .format_on_write()
            .merge(together)
            .await?;
    } else {
        assert!(all_in_one_imports_rx.is_empty());
    }
    info!("All done!");

    Ok(tf_work_dirs.into_iter().map(FreshTFWorkDir::from).collect())
}
