use crate::prelude::ProviderManager;
use cloud_terrastodon_core_command::prelude::CommandBuilder;
use cloud_terrastodon_core_command::prelude::CommandKind;
use cloud_terrastodon_core_command::prelude::CommandOutput;
use cloud_terrastodon_core_command::prelude::OutputBehaviour;
use cloud_terrastodon_core_command::prelude::bstr::BString;
use cloud_terrastodon_core_command::prelude::bstr::ByteSlice;
use cloud_terrastodon_core_command::prelude::bstr::io::BufReadExt;
use eyre::Context;
use eyre::OptionExt;
use eyre::Result;
use eyre::eyre;
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;
use tokio::fs;
use tracing::info;
use tracing::warn;

#[derive(Default)]
pub struct TofuGenerateConfigOutHelper {
    run_dir: Option<PathBuf>,
    plugin_dir: Option<PathBuf>,
}
impl TofuGenerateConfigOutHelper {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn with_run_dir(&mut self, dir: impl AsRef<Path>) -> &mut Self {
        self.run_dir = Some(dir.as_ref().to_path_buf());
        self
    }
    pub fn with_plugin_dir(&mut self, dir: impl AsRef<Path>) -> &mut Self {
        self.plugin_dir = Some(dir.as_ref().to_path_buf());
        self
    }
    #[track_caller]
    pub async fn run(&mut self) -> Result<()> {
        // Check preconditions
        let Some(ref work_dir) = self.run_dir else {
            return Err(eyre!("Run dir not set!"));
        };
        let result: eyre::Result<()> = (async ||{
            let provider_manager = ProviderManager::try_new()?;
            provider_manager.write_default_provider_configs(&work_dir).await?;
            // provider_manager.populate_provider_cache(&TofuTerraformRequiredProvidersBlock::common()).await?;

            // tf init
            let mut init_cmd = CommandBuilder::new(CommandKind::Tofu);
            init_cmd.should_announce(true);
            init_cmd.use_run_dir(work_dir);
            init_cmd.use_output_behaviour(OutputBehaviour::Display);
            // init_cmd.use_timeout(Duration::from_secs(120));
            init_cmd.args(["init", "-input=false"]);
            if let Some(plugin_dir) = &self.plugin_dir {
                init_cmd.arg(format!("-plugin-dir={}", plugin_dir.display()));
                // init_cmd.arg(plugin_dir);
            }
            init_cmd.run_raw().await?;
            info!("Tofu init successful!");

            // remove old plan outputs
            let generated_path = work_dir.join("generated.tf");
            if generated_path.exists() {
                if !generated_path.is_file() {
                    return Err(eyre!("generated output path exists but is not a file")
                        .wrap_err(generated_path.to_string_lossy().into_owned()));
                }
                fs::remove_file(generated_path).await?;
            }

            let mut validate_cmd = CommandBuilder::new(CommandKind::Tofu);
            validate_cmd.should_announce(true);
            validate_cmd.use_run_dir(work_dir);
            validate_cmd.use_output_behaviour(OutputBehaviour::Display);
            // validate_cmd.use_timeout(Duration::from_secs(30));
            validate_cmd.arg("validate");
            validate_cmd.run_raw().await?;

            // tf plan
            let mut plan_cmd = CommandBuilder::new(CommandKind::Tofu);
            plan_cmd.should_announce(true);
            plan_cmd.use_run_dir(work_dir.clone());
            plan_cmd.args([
                "plan",
                "-generate-config-out",
                "generated.tf",
                "-input=false",
            ]);

            info!("Executing import, please be patient.");
            let plan_result = plan_cmd.run_raw().await;
            match plan_result {
                Ok(_) => {
                    info!("Import success!");
                }
                Err(e) => {
                    let output = e
                        .downcast_ref::<CommandOutput>()
                        .ok_or_eyre("Failed to get command output details from error report")?;
                    let mut errors = Vec::new();
                    let needle_error_prefix =
                        "[31m│[0m [0m[1m[31mError: [0m[0m[1m".as_bytes();
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
                    for error in errors.iter().take(3) {
                        let error_text = BString::from(error[1].clone());
                        if let Some(error_text) = error_text
                            .strip_prefix(needle_error_prefix)
                            .and_then(|x| x.strip_suffix(needle_error_suffix))
                        {
                            seen_errors.insert(BString::from(error_text));
                        }
                    }
                    info!(
                        "Found {} errors ({} distinct)",
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
                            warn!("(auto-fixable) {}", error);
                        } else {
                            warn!("{}", error);
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
                }
            }

            // Success!
            info!("🚀 Successfully generated tf files from imports!");
            Ok(())
        })().await;
        result
            // .wrap_err(format!(
            //     "TofuImporter::run called from {}",
            //     std::panic::Location::caller()
            // ))
            .wrap_err(format!(
                "TofuImporter::run failed with dir \"{}\"",
                work_dir.display()
            ))
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use crate::prelude::ProviderManager;
    use crate::prelude::TofuGenerateConfigOutHelper;
    use cloud_terrastodon_core_pathing::AppDir;
    use cloud_terrastodon_core_pathing::Existy;
    use cloud_terrastodon_core_tofu_types::prelude::TofuTerraformRequiredProvidersBlock;
    use tempfile::Builder;
    use tokio::sync::Semaphore;
    use tokio::task::JoinSet;

    #[tokio::test]
    #[ignore]
    pub async fn terraform_concurrent_init_fail() -> eyre::Result<()> {
        let plugin_dir = ProviderManager::get_default_tf_plugin_cache_dir()?;
        println!(
            "This test can leave your plugin dir in a broken config, I recommend deleting {} after",
            plugin_dir.display()
        );
        let temp_dir = Builder::new().tempdir_in(AppDir::Temp.as_path_buf())?;
        let num_workspaces = 25;
        let mut join_set: JoinSet<eyre::Result<()>> = JoinSet::new();
        for i in 0..num_workspaces {
            let workspace_dir = temp_dir.path().join(format!("workspace_{i:03}"));
            join_set.spawn(async move {
                workspace_dir.ensure_dir_exists().await?;
                TofuGenerateConfigOutHelper::new()
                    .with_run_dir(&workspace_dir)
                    .run()
                    .await?;
                Ok(())
            });
        }
        while let Some(x) = join_set.join_next().await {
            x??;
        }
        Ok(())
    }

    #[tokio::test]
    #[ignore]
    pub async fn terraform_concurrent_init_happy() -> eyre::Result<()> {
        let temp_dir = Builder::new().tempdir_in(AppDir::Temp.as_path_buf())?;
        let num_workspaces = 25;
        let mut join_set: JoinSet<eyre::Result<()>> = JoinSet::new();

        let provider_manager = ProviderManager::try_new()?;
        provider_manager
            .populate_provider_cache(&TofuTerraformRequiredProvidersBlock::common())
            .await?;

        // let limit = Arc::new(Semaphore::new(1));
        let limit = Arc::new(Semaphore::new(num_workspaces));

        let cache_dir = provider_manager.local_mirror_dir;
        for i in 0..num_workspaces {
            let workspace_dir = temp_dir.path().join(format!("workspace_{i:03}"));
            let cache_dir = cache_dir.clone();
            let limit = limit.clone();
            join_set.spawn(async move {
                workspace_dir.ensure_dir_exists().await?;                
                let permit = limit.acquire().await?;
                TofuGenerateConfigOutHelper::new()
                    .with_run_dir(&workspace_dir)
                    .with_plugin_dir(cache_dir)
                    .run()
                    .await?;
                drop(permit);
                Ok(())
            });
        }

        _ = temp_dir.into_path(); // keep it around

        while let Some(x) = join_set.join_next().await {
            x??;
        }
        Ok(())
    }
}
