use cloud_terrastodon_core_pathing::AppDir;
use cloud_terrastodon_core_tofu::prelude::TofuGenerateConfigOutHelper;
use eyre::Result;
use std::path::PathBuf;
use tracing::info;

pub async fn perform_import() -> Result<()> {
    info!("Beginning tf import...");
    let imports_dir: PathBuf = AppDir::Imports.into();
    TofuGenerateConfigOutHelper::default()
        .with_run_dir(imports_dir)
        .run()
        .await?;

    Ok(())
}
