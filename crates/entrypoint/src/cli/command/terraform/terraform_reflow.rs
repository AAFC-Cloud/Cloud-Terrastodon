use clap::Args;
use eyre::Result;
use std::path::PathBuf;

/// Reflow generated Terraform source files.
#[derive(Args, Debug, Clone)]
pub struct TerraformReflowArgs {
    #[arg(default_value = ".")]
    pub source_dir: PathBuf,
    #[arg(
        long,
        default_value_t = false,
        help = "Recursively reflow source in subdirectories"
    )]
    pub recursive: bool,
}

impl TerraformReflowArgs {
    pub async fn invoke(self) -> Result<()> {
        // TODO: Implement reflow behavior.
        Ok(())
    }
}
