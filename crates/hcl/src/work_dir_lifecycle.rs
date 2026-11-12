use crate::ProviderManager;
use crate::discovery::DiscoveryDepth;
use crate::discovery::discover_hcl;
use cloud_terrastodon_command::CancellationReportExt;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::CommandOutput;
use cloud_terrastodon_command::bstr::BString;
use cloud_terrastodon_command::bstr::ByteSlice;
use cloud_terrastodon_command::bstr::io::BufReadExt;
use cloud_terrastodon_execution::ConcurrentIteratorExt;
use cloud_terrastodon_hcl_types::FreshTFWorkDir;
use cloud_terrastodon_hcl_types::GeneratedConfigOutTFWorkDir;
use cloud_terrastodon_hcl_types::HclBlock;
use cloud_terrastodon_hcl_types::InitializedTFWorkDir;
use cloud_terrastodon_hcl_types::IntoHclBlocks;
use cloud_terrastodon_hcl_types::ProviderSource;
use cloud_terrastodon_hcl_types::ProviderVersionConstraint;
use cloud_terrastodon_hcl_types::ProviderVersionObject;
use cloud_terrastodon_hcl_types::TerraformRequiredProvidersBlock;
use cloud_terrastodon_hcl_types::ValidatedTFWorkDir;
use eyre::Context;
use eyre::OptionExt;
use eyre::bail;
use hcl::edit::structure::Body;
use itertools::Itertools;
use rand::seq::SliceRandom;
use std::collections::HashSet;
use std::num::NonZeroUsize;
use std::path::Path;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use tokio::task::JoinSet;
use tracing::debug;
use tracing::info;
use tracing::warn;

pub async fn identify_required_providers(
    dir: impl AsRef<Path>,
) -> eyre::Result<TerraformRequiredProvidersBlock> {
    let body = discover_hcl(dir, DiscoveryDepth::Shallow)
        .await?
        .into_values()
        .flatten()
        .collect::<Body>();
    let blocks = body.try_into_hcl_blocks()?;
    let mut rtn = TerraformRequiredProvidersBlock::empty();
    for block in blocks {
        match block {
            HclBlock::Terraform(terraform_block) => {
                if let Some(required_providers) = terraform_block.required_providers {
                    for (key, version) in required_providers.0 {
                        rtn.try_merge_entry(key, version)?;
                    }
                }
            }
            HclBlock::Provider(provider_block) => {
                let provider_kind = provider_block.provider_kind();
                let provider_prefix = provider_kind.provider_prefix().to_string();
                let source: ProviderSource = provider_prefix.parse()?;
                let version = ProviderVersionObject {
                    source,
                    version: ProviderVersionConstraint::unspecified(),
                };
                rtn.try_merge_entry(provider_prefix, version)?;
            }
            HclBlock::Import(import_block) => {
                let provider_kind = import_block.to.provider_kind();
                let provider_prefix = provider_kind.provider_prefix().to_string();
                let source: ProviderSource = provider_prefix.parse()?;
                let version = ProviderVersionObject {
                    source,
                    version: ProviderVersionConstraint::unspecified(),
                };
                rtn.try_merge_entry(provider_prefix, version)?;
            }
            HclBlock::Resource(resource_block) => {
                let provider_kind = resource_block.provider_kind();
                let provider_prefix = provider_kind.provider_prefix().to_string();
                let source: ProviderSource = provider_prefix.parse()?;
                let version = ProviderVersionObject {
                    source,
                    version: ProviderVersionConstraint::unspecified(),
                };
                rtn.try_merge_entry(provider_prefix, version)?;
            }
            HclBlock::Data(data_block) => {
                let provider_kind = data_block.provider_kind();
                let provider_prefix = provider_kind.provider_prefix().to_string();
                let source: ProviderSource = provider_prefix.parse()?;
                let version = ProviderVersionObject {
                    source,
                    version: ProviderVersionConstraint::unspecified(),
                };
                rtn.try_merge_entry(provider_prefix, version)?;
            }
            HclBlock::Other(_block) => {}
        }
    }
    Ok(rtn)
}

pub async fn identify_required_providers_bulk(
    dirs: impl IntoIterator<Item = impl AsRef<Path>>,
) -> eyre::Result<TerraformRequiredProvidersBlock> {
    // These jobs only read and parse local HCL, with no commands, cache writes,
    // or required completion. Abandoning remaining reads on failure is safe.
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
    cancellation: &CancellationScope,
) -> eyre::Result<Vec<InitializedTFWorkDir>> {
    let dirs: Vec<FreshTFWorkDir> = dirs.into_iter().map(|x| x.into()).collect();
    info!("Initializing {} tf work dirs", dirs.len());
    let required_providers = identify_required_providers_bulk(&dirs).await?;
    let provider_manager = ProviderManager::try_new()?;
    provider_manager
        .populate_provider_cache(&required_providers, cancellation)
        .await?;

    let remaining = AtomicUsize::new(dirs.len());
    // Operation errors are data so initialization still attempts every directory.
    // Parent cancellation can stop optional work while started commands finish.
    dirs.into_iter().map_concurrent_scoped(
        cancellation,
        NonZeroUsize::new(5).ok_or_eyre("initialization concurrency must be nonzero")?,
        async |cancellation, dir| {
                let result = async {
                    let mut init_cmd = CommandBuilder::new(CommandKind::Terraform);
                    init_cmd.use_run_dir(&dir);
                    init_cmd.args(["init", "-input=false"]);
                    init_cmd.arg(format!(
                        "-plugin-dir={}",
                        provider_manager.local_mirror_dir.display().to_string().replace("\\", "/")
                    ));
                    init_cmd
                        .run_raw(&cancellation)
                        .await
                        .wrap_err("If you get a \"file in use\" warning it may be that a previous run has crashed but failed to kill child processes.\n```pwsh\nget-process -Name \"*terraform-provider*\" | Stop-Process\n```")?;
                    eyre::Ok(InitializedTFWorkDir::from(dir))
                }
                .await;
                info!(
                    "Initializing tf work dirs, {} remain...",
                    remaining.fetch_sub(1, Ordering::Relaxed) - 1
                );
                eyre::Ok(result)
        },
    )
    .await?
    .into_iter()
    .collect()
}

pub async fn validate_work_dirs(
    dirs: impl IntoIterator<Item = InitializedTFWorkDir>,
    cancellation: &CancellationScope,
) -> eyre::Result<Vec<ValidatedTFWorkDir>> {
    let dirs: Vec<InitializedTFWorkDir> = dirs.into_iter().collect();
    info!("Validating {} tf work dirs", dirs.len());
    let remaining = AtomicUsize::new(dirs.len());
    dirs.into_iter()
        .map_concurrent_scoped(
            cancellation,
            NonZeroUsize::new(16).ok_or_eyre("validation concurrency must be nonzero")?,
            async |cancellation, dir| {
                let mut validate_cmd = CommandBuilder::new(CommandKind::Terraform);
                validate_cmd.cancellation_safe();
                validate_cmd.use_run_dir(&dir);
                validate_cmd.arg("validate");
                let result = validate_cmd
                    .run_raw(&cancellation)
                    .await
                    .wrap_err_with(|| format!("Validating dir: {}", dir.display()));
                info!(
                    "Validating tf work dirs, {} remain...",
                    remaining.fetch_sub(1, Ordering::Relaxed) - 1
                );
                result?;
                eyre::Ok(ValidatedTFWorkDir::from(dir))
            },
        )
        .await
}

pub async fn generate_config_out(
    work_dir: &ValidatedTFWorkDir,
    cancellation: &CancellationScope,
) -> eyre::Result<()> {
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
    let plan_result = plan_cmd.run_raw(cancellation).await;
    let Err(e) = plan_result else {
        return Ok(());
    };
    if e.is_cancellation() {
        return Err(e);
    }
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
    cancellation: &CancellationScope,
) -> eyre::Result<Vec<GeneratedConfigOutTFWorkDir>> {
    let mut dirs = work_dirs.into_iter().collect_vec();
    dirs.shuffle(&mut rand::rng()); // try and hit errors earlier than if we had a consistent iteration order
    info!("Performing tf code generation for {} dirs", dirs.len());
    let remaining = AtomicUsize::new(dirs.len());
    // Preserve complete-all behavior for generation failures. Parent cancellation
    // still stops optional work and lets started commands retain their results.
    dirs.into_iter()
        .map_concurrent_scoped(
            cancellation,
            NonZeroUsize::new(10).ok_or_eyre("generation concurrency must be nonzero")?,
            async |cancellation, dir| {
                let result = generate_config_out(&dir, &cancellation)
                    .await
                    .map(|()| GeneratedConfigOutTFWorkDir::from(dir));
                info!(
                    "Performing tf code generation, {} dirs remain...",
                    remaining.fetch_sub(1, Ordering::Relaxed) - 1
                );
                eyre::Ok(result)
            },
        )
        .await?
        .into_iter()
        .collect()
}

#[cfg(test)]
mod test {
    use crate::identify_required_providers;
    use crate::identify_required_providers_bulk;
    use cloud_terrastodon_hcl_types::ProviderHostname;
    use cloud_terrastodon_hcl_types::ProviderKind;
    use cloud_terrastodon_hcl_types::ProviderNamespace;
    use cloud_terrastodon_hcl_types::ProviderSource;
    use cloud_terrastodon_hcl_types::ProviderVersionConstraint;
    use cloud_terrastodon_hcl_types::ProviderVersionObject;
    use cloud_terrastodon_hcl_types::TerraformRequiredProvidersBlock;
    use cloud_terrastodon_pathing::Existy;
    use indoc::indoc;

    #[tokio::test]
    pub async fn infer_providers() -> eyre::Result<()> {
        let temp_dir = tempfile::tempdir()?;
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
        let temp_dir = tempfile::tempdir()?;
        println!("Created temp dir at {}", temp_dir.path().display());

        let work_dir_1 = temp_dir.path().join("work_dir_1");
        let work_dir_2 = temp_dir.path().join("work_dir_2");
        let work_dir_3 = temp_dir.path().join("work_dir_3");
        for work_dir in [&work_dir_1, &work_dir_2, &work_dir_3] {
            work_dir.ensure_dir_exists().await?;
        }
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
        for (work_dir, content) in [
            (&work_dir_1, content_1),
            (&work_dir_2, content_2),
            (&work_dir_3, content_3),
        ] {
            tokio::fs::write(work_dir.join("main.tf"), content).await?;
        }
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
