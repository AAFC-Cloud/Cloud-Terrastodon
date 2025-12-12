use chrono::Local;
use clap::Args;
use cloud_terrastodon_azure::prelude::HclImportable;
use cloud_terrastodon_hcl::discovery::DiscoveryDepth;
use cloud_terrastodon_hcl::discovery::discover_hcl;
use cloud_terrastodon_hcl::prelude::GenerateConfigOutHelper;
use cloud_terrastodon_hcl::prelude::HclWriter;
use cloud_terrastodon_hcl::reflow::reflow_hcl;
use cloud_terrastodon_pathing::Existy;
use eyre::Result;
use std::path::PathBuf;
use tempfile::Builder;
use tracing::info;

/// Create Terraform import definitions for selected resources.
#[derive(Args, Debug, Clone)]
pub struct TerraformSourceGenerateArgs {
    #[arg(long, default_value = ".")]
    pub work_dir: PathBuf,
}

impl TerraformSourceGenerateArgs {
    pub async fn invoke(self) -> Result<()> {
        let work_dir = self.work_dir;

        let kind_to_import = HclImportable::pick()?;
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

        HclWriter::new(import_dir.join("imports.tf"))
            .format_on_write()
            .overwrite(imports)
            .await?;

        GenerateConfigOutHelper::new()
            .with_run_dir(&import_dir)
            .run()
            .await?;

        info!("Reflowing content");
        let hcl = discover_hcl(import_dir, DiscoveryDepth::Shallow).await?;
        let hcl = reflow_hcl(hcl).await?;
        for (path, contents) in hcl {
            HclWriter::new(path)
                .format_on_write()
                .overwrite(contents)
                .await?;
        }

        Ok(())
    }
}
