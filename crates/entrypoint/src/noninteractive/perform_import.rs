use cloud_terrastodon_hcl::prelude::GenerateConfigOutHelper;
use cloud_terrastodon_hcl::prelude::ProviderManager;
use cloud_terrastodon_pathing::AppDir;
use eyre::Result;
use std::path::PathBuf;
use tracing::info;

pub async fn perform_import() -> Result<()> {
    info!("Beginning tf import...");
    let imports_dir: PathBuf = AppDir::Imports.into();
    ProviderManager::try_new()?
        .write_default_provider_configs(&imports_dir)
        .await?;
    GenerateConfigOutHelper::default()
        .with_run_dir(imports_dir)
        .run()
        .await?;

    Ok(())
}
