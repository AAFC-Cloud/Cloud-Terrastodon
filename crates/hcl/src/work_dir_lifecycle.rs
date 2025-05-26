use crate::prelude::ProviderManager;
use crate::reflow::as_single_body;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::CommandOutput;
use cloud_terrastodon_command::bstr::BString;
use cloud_terrastodon_command::bstr::ByteSlice;
use cloud_terrastodon_command::bstr::io::BufReadExt;
use cloud_terrastodon_hcl_types::prelude::FreshTFWorkDir;
use cloud_terrastodon_hcl_types::prelude::GeneratedConfigOutTFWorkDir;
use cloud_terrastodon_hcl_types::prelude::InitializedTFWorkDir;
use cloud_terrastodon_hcl_types::prelude::IntoHCLBlocks;
use cloud_terrastodon_hcl_types::prelude::ProviderSource;
use cloud_terrastodon_hcl_types::prelude::ProviderVersionConstraint;
use cloud_terrastodon_hcl_types::prelude::HCLBlock;
use cloud_terrastodon_hcl_types::prelude::ProviderVersionObject;
use cloud_terrastodon_hcl_types::prelude::TerraformRequiredProvidersBlock;
use cloud_terrastodon_hcl_types::prelude::ValidatedTFWorkDir;
use eyre::Context;
use eyre::OptionExt;
use eyre::bail;
use itertools::Itertools;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;
use tracing::debug;
use tracing::info;
use tracing::warn;

pub async fn identify_required_providers(
    dir: impl AsRef<Path>,
) -> eyre::Result<TerraformRequiredProvidersBlock> {
    let body = as_single_body(dir).await?;
    let blocks = body.try_into_hcl_blocks()?;
    let mut rtn = TerraformRequiredProvidersBlock::empty();
    for block in blocks {
        match block {
            HCLBlock::Terraform(terraform_block) => {
                if let Some(required_providers) = terraform_block.required_providers {
                    for (key, version) in required_providers.0 {
                        rtn.try_merge_entry(key, version)?;
                    }
                }
            }
            HCLBlock::Provider(provider_block) => {
                let provider_kind = provider_block.provider_kind();
                let provider_prefix = provider_kind.provider_prefix().to_string();
                let source: ProviderSource = provider_prefix.parse()?;
                let version = ProviderVersionObject {
                    source,
                    version: ProviderVersionConstraint::unspecified(),
                };
                rtn.try_merge_entry(provider_prefix, version)?;
            }
            HCLBlock::Import(import_block) => {
                let provider_kind = import_block.to.provider_kind();
                let provider_prefix = provider_kind.provider_prefix().to_string();
                let source: ProviderSource = provider_prefix.parse()?;
                let version = ProviderVersionObject {
                    source,
                    version: ProviderVersionConstraint::unspecified(),
                };
                rtn.try_merge_entry(provider_prefix, version)?;
            }
            HCLBlock::Other(_block) => {}
        }
    }
    Ok(rtn)
}

pub async fn identify_required_providers_bulk(
    dirs: impl IntoIterator<Item = impl AsRef<Path>>,
) -> eyre::Result<TerraformRequiredProvidersBlock> {
    let mut join_set: JoinSet<eyre::Result<TerraformRequiredProvidersBlock>> = JoinSet::new();
    for dir in dirs {
        let dir = dir.as_ref().to_path_buf();
        join_set.spawn(async move { identify_required_providers(dir).await });
    }
    let mut rtn = Vec::with_capacity(join_set.len());
    while let Some(x) = join_set.join_next().await {
        rtn.push(x??);
        info!(
            "Identifying required providers, {} work dirs remain",
            join_set.len()
        );
    }
    let rtn = TerraformRequiredProvidersBlock::try_from_iter(rtn)?;
    Ok(rtn)
}

pub async fn initialize_work_dirs(
    dirs: impl IntoIterator<Item = impl Into<FreshTFWorkDir>>,
) -> eyre::Result<Vec<InitializedTFWorkDir>> {
    let dirs: Vec<FreshTFWorkDir> = dirs.into_iter().map(|x| x.into()).collect();
    info!("Initializing {} tf work dirs", dirs.len());
    let required_providers = identify_required_providers_bulk(&dirs).await?;
    let provider_manager = ProviderManager::try_new()?;
    provider_manager
        .populate_provider_cache(&required_providers)
        .await?;

    let mut join_set: JoinSet<eyre::Result<()>> = JoinSet::new();
    let parallism = Arc::new(Semaphore::new(5));
    for dir in &dirs {
        let plugin_dir = provider_manager.local_mirror_dir.clone();
        let dir = dir.clone();
        let parallelism = parallism.clone();
        join_set.spawn(async move {
            let permit = parallelism.acquire().await?;
            let mut init_cmd = CommandBuilder::new(CommandKind::Terraform);
            init_cmd.use_run_dir(dir);
            init_cmd.args(["init", "-input=false"]);
            init_cmd.arg(format!(
                "-plugin-dir={}",
                plugin_dir.display().to_string().replace("\\", "/")
            ));
            init_cmd.run_raw().await.wrap_err("If you get a \"file in use\" warning it may be that a previous run has crashed but failed to kill child processes.\n```pwsh\nget-process -Name \"*terraform-provider*\" | Stop-Process\n```")?;
            drop(permit);
            Ok(())
        });
    }

    while let Some(x) = join_set.join_next().await {
        x??;
        info!("Initializing tf work dirs, {} remain...", join_set.len());
    }

    Ok(dirs.into_iter().map(InitializedTFWorkDir::from).collect())
}

pub async fn validate_work_dirs(
    dirs: impl IntoIterator<Item = InitializedTFWorkDir>,
) -> eyre::Result<Vec<ValidatedTFWorkDir>> {
    let dirs: Vec<InitializedTFWorkDir> = dirs.into_iter().collect();
    info!("Validating {} tf work dirs", dirs.len());
    let mut join_set: JoinSet<eyre::Result<()>> = JoinSet::new();
    let rate_limit = Arc::new(Semaphore::new(16));
    for dir in &dirs {
        let dir = dir.clone();
        let rate_limit = rate_limit.clone();
        join_set.spawn(async move {
            let mut validate_cmd = CommandBuilder::new(CommandKind::Terraform);
            // validate_cmd.should_announce(true);
            validate_cmd.use_run_dir(&dir);
            validate_cmd.arg("validate");
            let permit = rate_limit.acquire().await?;
            validate_cmd
                .run_raw()
                .await
                .wrap_err(format!("Validating dir: {}", dir.display()))?;
            drop(permit);
            Ok(())
        });
    }

    while let Some(x) = join_set.join_next().await {
        x??;
        info!("Validating tf work dirs, {} remain...", join_set.len());
    }

    Ok(dirs.into_iter().map(ValidatedTFWorkDir::from).collect())
}

pub async fn generate_config_out(work_dir: &ValidatedTFWorkDir) -> eyre::Result<()> {
    debug!("Performing tf generation from {}", work_dir.display());
    let mut plan_cmd = CommandBuilder::new(CommandKind::Terraform);
    // plan_cmd.should_announce(true);
    plan_cmd.use_run_dir(work_dir.clone());
    plan_cmd.args([
        "plan",
        "-generate-config-out",
        "generated.tf",
        "-input=false",
    ]);
    let plan_result = plan_cmd.run_raw().await;
    let Err(e) = plan_result else {
        return Ok(());
    };
    let output = e
        .downcast_ref::<CommandOutput>()
        .ok_or_eyre("Failed to get command output details from error report")?;
    let mut errors = Vec::new();
    let needle_error_prefix = "[31m│[0m [0m[1m[31mError: [0m[0m[1m".as_bytes();
    let needle_error_suffix = "[0m".as_bytes();
    let needle_error_end = "[31m╵[0m[0m".as_bytes();
    let mut lines_buffer = Vec::new();
    for line in output.stderr.byte_lines() {
        let line = line?;
        if line == needle_error_end {
            lines_buffer.push(line);
            errors.push(lines_buffer);
            lines_buffer = Vec::new();
        } else {
            lines_buffer.push(line);
        }
    }
    let mut seen_errors = HashSet::new();
    for error in &errors {
        let error_text = BString::from(error[1].clone());
        let Some(error_text) = error_text
            .strip_prefix(needle_error_prefix)
            .map(|x| x.strip_suffix(needle_error_suffix).unwrap_or(x))
        else {
            bail!("Unable to extract error text from error:\n{error_text}");
        };
        seen_errors.insert(BString::from(error_text));
    }
    debug!(
        "Failed to generate tf code from {}, found {} errors ({} distinct)",
        work_dir.display(),
        errors.len(),
        seen_errors.len()
    );
    let fixable_errors: HashSet<BString> = HashSet::from_iter([
        BString::from("Insufficient initialization blocks"),
        BString::from("Feature map must contain at least on entry"),
    ]);
    let mut unfixable_error_count = 0;
    for error in seen_errors {
        if fixable_errors.contains(&error) {
            debug!("Extracted error (auto-fixable): {}", error);
        } else {
            debug!("Extracted error: {}", error);
            if error.contains_str("No valid credentials found") {
                warn!(
                    "Did you forget to set your devops access token?\n```pwsh\n$env:AZDO_PERSONAL_ACCESS_TOKEN=Read-Host -MaskInput \"Enter PAT\"\n```"
                );
            }
            unfixable_error_count += 1;
        }
    }
    if unfixable_error_count > 0 {
        return Err(e.wrap_err(format!(
            "Errors present during import, found {unfixable_error_count} errors that are not fixable by the fixer-upper.",
        )));
    }
    Ok(())
}

pub async fn generate_config_out_bulk(
    work_dirs: impl IntoIterator<Item = ValidatedTFWorkDir>,
) -> eyre::Result<Vec<GeneratedConfigOutTFWorkDir>> {
    let mut dirs = work_dirs.into_iter().collect_vec();
    dirs.shuffle(&mut thread_rng()); // try and hit errors earlier than if we had a consistent iteration order
    info!("Performing tf code generation for {} dirs", dirs.len());
    let mut join_set: JoinSet<eyre::Result<()>> = JoinSet::new();
    let rate_limit = Arc::new(Semaphore::new(10));
    for dir in &dirs {
        let rate_limit = rate_limit.clone();
        let dir = dir.clone();
        join_set.spawn(async move {
            let permit = rate_limit.acquire().await?;
            generate_config_out(&dir).await?;
            drop(permit);
            Ok(())
        });
    }

    while let Some(x) = join_set.join_next().await {
        x??;
        info!(
            "Performing tf code generation, {} dirs remain...",
            join_set.len()
        );
    }
    Ok(dirs.into_iter().map(|x| x.into()).collect())
}

#[cfg(test)]
mod test {
    use crate::prelude::ProviderManager;
    use crate::prelude::generate_config_out_bulk;
    use crate::prelude::identify_required_providers;
    use crate::prelude::identify_required_providers_bulk;
    use crate::prelude::initialize_work_dirs;
    use crate::prelude::validate_work_dirs;
    use cloud_terrastodon_pathing::AppDir;
    use cloud_terrastodon_pathing::Existy;
    use cloud_terrastodon_hcl_types::prelude::ProviderHostname;
    use cloud_terrastodon_hcl_types::prelude::ProviderNamespace;
    use cloud_terrastodon_hcl_types::prelude::ProviderSource;
    use cloud_terrastodon_hcl_types::prelude::ProviderVersionConstraint;
    use cloud_terrastodon_hcl_types::prelude::ProviderKind;
    use cloud_terrastodon_hcl_types::prelude::ProviderVersionObject;
    use cloud_terrastodon_hcl_types::prelude::TerraformRequiredProvidersBlock;
    use indoc::indoc;
    use tokio::try_join;

    fn init_logging() -> eyre::Result<()> {
        let env_filter = tracing_subscriber::EnvFilter::builder()
            .with_default_directive(tracing::level_filters::LevelFilter::DEBUG.into())
            .from_env_lossy();
        tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .with_file(true)
            .with_line_number(true)
            .without_time()
            .init();
        Ok(())
    }

    #[tokio::test]
    #[ignore]
    pub async fn terraform_concurrent_init_happy() -> eyre::Result<()> {
        init_logging()?;
        let temp_dir = tempfile::Builder::new().tempdir_in(AppDir::Temp.as_path_buf())?;
        println!("Check out the dirs at {}", temp_dir.path().display());
        let num_workspaces = 25;

        let provider_manager = ProviderManager::try_new()?;

        let mut work_dirs = Vec::new();
        for i in 0..num_workspaces {
            let workspace_dir = temp_dir.path().join(format!("workspace_{i:03}"));
            provider_manager
                .write_default_provider_configs(&workspace_dir)
                .await?;
            let content = indoc! {r#"
                import {
                    id = "123,0,1000"
                    to = random_integer.bruh
                }
            "#};
            tokio::fs::write(workspace_dir.join("main.tf"), content).await?;
            work_dirs.push(workspace_dir)
        }

        println!("Initializing");
        let work_dirs = initialize_work_dirs(work_dirs).await?;
        println!("Validating");
        let work_dirs = validate_work_dirs(work_dirs).await?;
        println!("Generating");
        generate_config_out_bulk(work_dirs).await?;

        println!("Check out the dirs at {}", temp_dir.path().display());
        _ = temp_dir.into_path(); // keep it around
        Ok(())
    }

    #[tokio::test]
    pub async fn infer_providers() -> eyre::Result<()> {
        let temp_dir = tempfile::Builder::new().tempdir_in(AppDir::Temp.as_path_buf())?;
        println!("Check out the dirs at {}", temp_dir.path().display());
        let content = indoc! {r#"
            import {
                id = "123,0,1000"
                to = random_integer.bruh
            }
        "#};
        tokio::fs::write(temp_dir.path().join("main.tf"), content).await?;
        let required_providers = identify_required_providers(temp_dir.path()).await?;
        dbg!(&required_providers);

        assert_eq!(
            required_providers,
            TerraformRequiredProvidersBlock(
                [(
                    "random".to_string(),
                    ProviderVersionObject {
                        source: ProviderSource {
                            hostname: ProviderHostname::default(),
                            namespace: ProviderNamespace::default(),
                            kind: ProviderKind::Other("random".to_string())
                        },
                        version: ProviderVersionConstraint::unspecified()
                    }
                )]
                .into()
            )
        );

        println!("Check out the dirs at {}", temp_dir.path().display());
        // _ = temp_dir.into_path(); // keep it around
        Ok(())
    }

    #[tokio::test]
    pub async fn infer_providers_bulk() -> eyre::Result<()> {
        let temp_dir = tempfile::Builder::new().tempdir_in(AppDir::Temp.as_path_buf())?;
        println!("Created temp dir at {}", temp_dir.path().display());

        let work_dir_1 = temp_dir.path().join("work_dir_1");
        let work_dir_2 = temp_dir.path().join("work_dir_2");
        let work_dir_3 = temp_dir.path().join("work_dir_3");
        try_join!(
            work_dir_1.ensure_dir_exists(),
            work_dir_2.ensure_dir_exists(),
            work_dir_3.ensure_dir_exists(),
        )?;
        let content_1 = indoc! {r#"
            import {
                id = "123,0,1000"
                to = random_integer.bruh
            }
        "#};
        let content_2 = indoc! {r#"
            terraform {
                required_providers {
                    azurerm = {
                        source = "hashicorp/azurerm"
                    }
                }
            }
        "#};
        let content_3 = indoc! {r#"
            import {
                id = "someidherelol"
                to = azuread_group.bruh
            }
        "#};
        try_join!(
            tokio::fs::write(work_dir_1.join("main.tf"), content_1),
            tokio::fs::write(work_dir_2.join("main.tf"), content_2),
            tokio::fs::write(work_dir_3.join("main.tf"), content_3),
        )?;
        let required_providers =
            identify_required_providers_bulk([work_dir_1, work_dir_2, work_dir_3]).await?;
        dbg!(&required_providers);

        assert_eq!(
            required_providers,
            TerraformRequiredProvidersBlock(
                [
                    (
                        "random".to_string(),
                        ProviderVersionObject {
                            source: ProviderSource {
                                hostname: ProviderHostname::default(),
                                namespace: ProviderNamespace::default(),
                                kind: ProviderKind::Other("random".to_string())
                            },
                            version: ProviderVersionConstraint::unspecified()
                        }
                    ),
                    (
                        "azurerm".to_string(),
                        ProviderVersionObject {
                            source: ProviderSource {
                                hostname: ProviderHostname::default(),
                                namespace: ProviderNamespace::default(),
                                kind: ProviderKind::AzureRM
                            },
                            version: ProviderVersionConstraint::unspecified()
                        }
                    ),
                    (
                        "azuread".to_string(),
                        ProviderVersionObject {
                            source: ProviderSource {
                                hostname: ProviderHostname::default(),
                                namespace: ProviderNamespace::default(),
                                kind: ProviderKind::AzureAD
                            },
                            version: ProviderVersionConstraint::unspecified()
                        }
                    )
                ]
                .into()
            )
        );

        // _ = temp_dir.into_path(); // keep it around
        Ok(())
    }
}
