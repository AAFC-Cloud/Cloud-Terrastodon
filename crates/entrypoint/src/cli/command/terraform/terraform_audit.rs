use clap::Args;
use cloud_terrastodon_hcl::prelude::discover_terraform_source_dirs;
use eyre::Result;
use std::path::PathBuf;

/// Identify and report Terraform provider issues.
#[derive(Args, Debug, Clone)]
pub struct TerraformAuditArgs {
    #[arg(default_value = ".")]
    pub source_dir: PathBuf,
    #[arg(
        long,
        default_value_t = false,
        help = "Recursively audit subdirectories"
    )]
    pub recursive: bool,
}

impl TerraformAuditArgs {
    pub async fn invoke(self) -> Result<()> {
        if self.recursive {
            let source_dirs = discover_terraform_source_dirs(self.source_dir).await?;
            for dir in source_dirs {
                cloud_terrastodon_hcl::prelude::audit(&dir).await?;
            }
        } else {
            cloud_terrastodon_hcl::prelude::audit(&self.source_dir).await?;
        }

        Ok(())
    }
}
