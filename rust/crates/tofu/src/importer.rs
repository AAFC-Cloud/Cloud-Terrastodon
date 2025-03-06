use cloud_terrastodon_core_azure_devops::prelude::get_default_organization_name;
use cloud_terrastodon_core_command::prelude::CommandBuilder;
use cloud_terrastodon_core_command::prelude::CommandKind;
use cloud_terrastodon_core_command::prelude::OutputBehaviour;
use cloud_terrastodon_core_tofu_types::prelude::TofuProviderBlock;
use cloud_terrastodon_core_tofu_types::prelude::TofuTerraformBlock;
use cloud_terrastodon_core_tofu_types::prelude::TofuTerraformProviderVersionObject;
use cloud_terrastodon_core_tofu_types::prelude::TofuTerraformRequiredProvidersBlock;
use eyre::Context;
use eyre::Result;
use eyre::eyre;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;
use tokio::fs;
use tracing::info;

use crate::prelude::TofuWriter;

#[derive(Default)]
pub struct TofuImporter {
    imports_dir: Option<PathBuf>,
}
impl TofuImporter {
    pub fn using_dir(&mut self, imports_dir: impl AsRef<Path>) -> &mut Self {
        self.imports_dir = Some(imports_dir.as_ref().to_path_buf());
        self
    }
    pub async fn run(&mut self) -> Result<()> {
        // Check preconditions
        let Some(ref imports_dir) = self.imports_dir else {
            return Err(eyre!("Dir must be set with using_dir"));
        };

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
                TofuProviderBlock::AzureRM {
                    alias: None,
                    subscription_id: None,
                },
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
        init_cmd.use_timeout(Duration::from_secs(120));
        init_cmd.arg("init");
        init_cmd.run_raw().await?;
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
        validate_cmd.use_timeout(Duration::from_secs(30));
        validate_cmd.arg("validate");
        validate_cmd.run_raw().await?;

        // tf plan
        let mut plan_cmd = CommandBuilder::new(CommandKind::Tofu);
        plan_cmd.should_announce(true);
        plan_cmd.use_run_dir(imports_dir.clone());
        plan_cmd.args(["plan", "-generate-config-out", "generated.tf"]);

        info!("Executing import, please be patient.");
        plan_cmd.run_raw().await?;
        info!("Tofu plan successful!");

        // Success!
        info!("🚀 Successfully generated tofu files from imports!");
        Ok(())
    }
}
