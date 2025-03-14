use cloud_terrastodon_core_azure_devops::prelude::get_default_organization_name;
use cloud_terrastodon_core_command::prelude::CommandBuilder;
use cloud_terrastodon_core_command::prelude::CommandKind;
use cloud_terrastodon_core_command::prelude::CommandOutput;
use cloud_terrastodon_core_command::prelude::OutputBehaviour;
use cloud_terrastodon_core_command::prelude::bstr::BString;
use cloud_terrastodon_core_command::prelude::bstr::ByteSlice;
use cloud_terrastodon_core_command::prelude::bstr::io::BufReadExt;
use cloud_terrastodon_core_tofu_types::prelude::TofuProviderBlock;
use cloud_terrastodon_core_tofu_types::prelude::TofuTerraformBlock;
use cloud_terrastodon_core_tofu_types::prelude::TofuTerraformProviderVersionObject;
use cloud_terrastodon_core_tofu_types::prelude::TofuTerraformRequiredProvidersBlock;
use eyre::Context;
use eyre::OptionExt;
use eyre::Result;
use eyre::eyre;
use std::any::type_name;
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;
use tokio::fs;
use tracing::info;
use tracing::warn;

use crate::prelude::TofuWriter;

#[derive(Default)]
pub struct TofuImporter {
    imports_dir: Option<PathBuf>,
}
impl TofuImporter {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn using_dir(&mut self, imports_dir: impl AsRef<Path>) -> &mut Self {
        self.imports_dir = Some(imports_dir.as_ref().to_path_buf());
        self
    }
    pub async fn run(&mut self) -> Result<()> {
        // Check preconditions
        let Some(ref imports_dir) = self.imports_dir else {
            return Err(eyre!("Dir must be set with using_dir"));
        };
        let result: eyre::Result<()> = try {
            // Get devops url
            let org_service_url = format!(
                "https://dev.azure.com/{name}/",
                name = get_default_organization_name().await?
            );

            // Open boilerplate file
            info!("Writing default providers");
            let boilerplate_path = imports_dir.join("boilerplate.tf");
            let import_writer = TofuWriter::new(boilerplate_path);
            import_writer
                .merge(vec![TofuTerraformBlock {
                    required_providers: Some(TofuTerraformRequiredProvidersBlock(
                        [
                            (
                                "azurerm".to_string(),
                                TofuTerraformProviderVersionObject {
                                    source: "hashicorp/azurerm".to_string(),
                                    version: ">=4.18.0".to_string(),
                                },
                            ),
                            (
                                "azuread".to_string(),
                                TofuTerraformProviderVersionObject {
                                    source: "hashicorp/azuread".to_string(),
                                    version: ">=3.1.0".to_string(),
                                },
                            ),
                            (
                                "azuredevops".to_string(),
                                TofuTerraformProviderVersionObject {
                                    source: "microsoft/azuredevops".to_string(),
                                    version: ">=1.6.0".to_string(),
                                },
                            ),
                        ]
                        .into(),
                    )),
                    ..Default::default()
                }])
                .await
                .context("writing terraform block")?
                .merge(vec![
                    // TofuProviderBlock::AzureRM {
                    //     alias: None,
                    //     subscription_id: Some(""),
                    // },
                    TofuProviderBlock::AzureDevOps {
                        alias: None,
                        org_service_url,
                    },
                ])
                .await
                .context("writing default azurerm provider block")?
                .format()
                .await?;

            // tf init
            let mut init_cmd = CommandBuilder::new(CommandKind::Tofu);
            init_cmd.should_announce(true);
            init_cmd.use_run_dir(imports_dir);
            init_cmd.use_output_behaviour(OutputBehaviour::Display);
            // init_cmd.use_timeout(Duration::from_secs(120));
            init_cmd.args(["init", "-input","false"]);
            init_cmd.run_raw().await.context("performing tf init")?;
            info!("Tofu init successful!");

            // remove old plan outputs
            let generated_path = imports_dir.join("generated.tf");
            if generated_path.exists() {
                if !generated_path.is_file() {
                    return Err(eyre!("generated output path exists but is not a file")
                        .wrap_err(generated_path.to_string_lossy().into_owned()));
                }
                fs::remove_file(generated_path).await?;
            }

            let mut validate_cmd = CommandBuilder::new(CommandKind::Tofu);
            validate_cmd.should_announce(true);
            validate_cmd.use_run_dir(imports_dir);
            validate_cmd.use_output_behaviour(OutputBehaviour::Display);
            // validate_cmd.use_timeout(Duration::from_secs(30));
            validate_cmd.arg("validate");
            validate_cmd
                .run_raw()
                .await
                .context("performing tf validate")?;

            // tf plan
            let mut plan_cmd = CommandBuilder::new(CommandKind::Tofu);
            plan_cmd.should_announce(true);
            plan_cmd.use_run_dir(imports_dir.clone());
            plan_cmd.args(["plan", "-generate-config-out", "generated.tf", "-input","false"]);

            info!("Executing import, please be patient.");
            let plan_result = plan_cmd.run_raw().await.context("performing tf plan");
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
        };
        result.wrap_err(format!(
            "Performing using {} \"{}\" against dir \"{}\"",
            type_name::<TofuImporter>(),
            file!(),
            imports_dir.display()
        ))
    }
}
