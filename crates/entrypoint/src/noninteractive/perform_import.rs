use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_hcl::GenerateConfigOutHelper;
use cloud_terrastodon_hcl::ProviderManager;
use cloud_terrastodon_pathing::AppDir;
use eyre::Result;
use std::path::PathBuf;
use tracing::info;

pub async fn perform_import(
    cancellation: &CancellationScope,
) -> Result<()> {
    info!("Beginning tf import...");
    let imports_dir: PathBuf = AppDir::Imports.into();
    ProviderManager::try_new()?
        .write_default_provider_configs(&imports_dir, cancellation)
        .await?;
    GenerateConfigOutHelper::default()
        .with_run_dir(imports_dir)
        .run(cancellation)
        .await?;

    Ok(())
}
