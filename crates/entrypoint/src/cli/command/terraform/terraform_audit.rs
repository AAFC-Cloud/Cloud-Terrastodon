use cloud_terrastodon_hcl::discover_terraform_source_dirs;
use eyre::Result;
use std::path::PathBuf;

/// Identify and report Terraform provider issues.
#[derive(facet::Facet, Debug, Clone)]
pub struct TerraformAuditArgs {
    #[facet(figue::positional, default = PathBuf::from("."))]
    pub source_dir: PathBuf,
    #[facet(figue::named, default = false)]
    pub recursive: bool,
}

impl TerraformAuditArgs {
    pub async fn invoke(self) -> Result<()> {
        if self.recursive {
            let source_dirs = discover_terraform_source_dirs(self.source_dir).await?;
            for dir in source_dirs {
                cloud_terrastodon_hcl::audit(&dir).await?;
            }
        } else {
            cloud_terrastodon_hcl::audit(&self.source_dir).await?;
        }

        Ok(())
    }
}
