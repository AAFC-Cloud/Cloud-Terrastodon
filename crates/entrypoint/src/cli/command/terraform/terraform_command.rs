use chrono::Local;
use clap::Subcommand;
use cloud_terrastodon_azure::prelude::HCLImportable;
use cloud_terrastodon_hcl::prelude::GenerateConfigOutHelper;
use cloud_terrastodon_hcl::prelude::HCLWriter;
use cloud_terrastodon_hcl::prelude::discover_terraform_source_dirs;
use cloud_terrastodon_hcl::prelude::reflow_workspace;
use cloud_terrastodon_pathing::Existy;
use eyre::Result;
use std::path::PathBuf;
use tempfile::Builder;
use tracing::info;

/// Terraform-specific commands.
#[derive(Subcommand, Debug, Clone)]
pub enum TerraformCommand {
    /// Create Terraform import definitions for selected resources.
    Import {
        #[arg(long, default_value = ".")]
        work_dir: PathBuf,
    },
    /// Identify and report Terraform provider issues.
    ///
    /// Identify if any providers have been specified as required but are not being used.
    ///
    /// Identify if any providers are not using the latest version.
    Audit {
        #[arg(default_value = ".")]
        source_dir: PathBuf,
        #[arg(
            long,
            default_value_t = false,
            help = "Recursively audit subdirectories"
        )]
        recursive: bool,
    },
}

impl TerraformCommand {
    pub async fn invoke(self) -> Result<()> {
        match self {
            TerraformCommand::Import { work_dir } => {
                let kind_to_import = HCLImportable::pick()?;
                let imports = kind_to_import.pick_into_body().await?;

                work_dir.ensure_dir_exists().await?;
                let temp_dir = Builder::new()
                    .prefix(&format!(
                        "generated_{}",
                        Local::now().format("%Y%m%d_%H%M%S_")
                    ))
                    .suffix(".tf")
                    .tempdir_in(&work_dir)?;
                let import_dir = temp_dir.keep();

                HCLWriter::new(import_dir.join("imports.tf"))
                    .format_on_write()
                    .overwrite(imports)
                    .await?;

                GenerateConfigOutHelper::new()
                    .with_run_dir(&import_dir)
                    .run()
                    .await?;

                info!("Reflowing content");
                let files = reflow_workspace(&import_dir)
                    .await?
                    .get_file_contents(&import_dir)?;
                for (path, contents) in files {
                    HCLWriter::new(path)
                        .format_on_write()
                        .overwrite(contents)
                        .await?;
                }
            }
            TerraformCommand::Audit {
                source_dir,
                recursive,
            } => {
                if recursive {
                    let source_dirs = discover_terraform_source_dirs(source_dir).await?;
                    for dir in source_dirs {
                        cloud_terrastodon_hcl::prelude::audit(&dir).await?;
                    }
                } else {
                    cloud_terrastodon_hcl::prelude::audit(&source_dir).await?;
                }
            }
        }

        Ok(())
    }
}
