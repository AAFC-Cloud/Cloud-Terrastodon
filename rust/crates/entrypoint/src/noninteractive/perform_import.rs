use cloud_terrastodon_core_pathing::AppDir;
use cloud_terrastodon_core_tofu::prelude::TofuImporter;
use eyre::Result;
use std::path::PathBuf;
use tracing::info;

pub async fn perform_import() -> Result<()> {
    info!("Beginning tf import...");
    let imports_dir: PathBuf = AppDir::Imports.into();
    TofuImporter::default().using_dir(imports_dir).run().await?;

    Ok(())
}
