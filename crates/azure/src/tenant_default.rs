use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_pathing::AppDir;
use eyre::WrapErr;

/// Read the tenant deliberately configured by `ct az tenant default`.
/// Account admission is separate and never consults Azure CLI's active tenant.
pub async fn get_default_tenant_id(
    cancellation: &CancellationScope,
) -> eyre::Result<AzureTenantId> {
    let text = tokio::fs::read_to_string(AppDir::Tenants.as_path_buf().join("default.txt"))
        .await
        .wrap_err("no readable Cloud Terrastodon tenant default; specify --tenant or configure `ct az tenant default <tenant>`")?;
    cancellation.bail_if_cancelled()?;
    text.trim()
        .parse()
        .wrap_err("parsing configured Cloud Terrastodon tenant default")
}

/// Persist an explicit tenant default without selecting an account or acquiring credentials.
pub async fn set_default_tenant_id(
    tenant: AzureTenantId,
    cancellation: &CancellationScope,
) -> eyre::Result<()> {
    cancellation
        .deferred("persist tenant default", |_protected| async move {
            let root = AppDir::Tenants.as_path_buf();
            tokio::fs::create_dir_all(&root).await?;
            tokio::fs::write(root.join("default.txt"), format!("{tenant}\n")).await?;
            Ok(())
        })
        .await
}
