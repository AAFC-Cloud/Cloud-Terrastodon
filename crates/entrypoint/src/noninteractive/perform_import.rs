use cloud_terrastodon_pathing::AppDir;
use cloud_terrastodon_hcl::prelude::GenerateConfigOutHelper;
use eyre::Result;
use std::path::PathBuf;
use tracing::info;

pub async fn perform_import() -> Result<()> {
    info!("Beginning tf import...");
    let imports_dir: PathBuf = AppDir::Imports.into();
    GenerateConfigOutHelper::default()
        .with_run_dir(imports_dir)
        .run()
        .await?;

    Ok(())
}
