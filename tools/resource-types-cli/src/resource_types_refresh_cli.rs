use crate::ResourceTypeCatalogue;
use crate::ResourceTypesRefreshAuthSourceArgument;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_app::Result;
use cloud_terrastodon_app::facet;
use cloud_terrastodon_app::figue;
use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::fetch_all_resource_providers;
use cloud_terrastodon_azure_types::AzureTenantArgument;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::WrapErr;
use eyre::ensure;
use facet::Facet;
use std::fs;
use std::io::ErrorKind;
use std::path::Path;
use std::path::PathBuf;

/// Save the tenant's resource-type vocabulary from Azure's provider metadata.
///
/// Uses the Azure SDK's tenant-scoped provider request. Only normalized
/// resource-type names are saved; provider metadata and credential/runtime state
/// are not part of the catalogue.
#[derive(Facet)]
pub struct ResourceTypesRefreshArgs {
    /// Tracked tenant UUID or alias. Omit or use default for the Azure CLI default tenant.
    #[facet(figue::named, default)]
    pub tenant: AzureTenantArgument<'static>,
    /// Explicit azure-cli, browser, or workload-identity credential source.
    #[facet(figue::named)]
    pub auth_source: ResourceTypesRefreshAuthSourceArgument,
    /// Destination JSON catalogue; defaults to catalogue.json beside the tool's Cargo.toml.
    /// Its parent directory must already exist.
    #[facet(figue::named, default = ResourceTypeCatalogue::default_path())]
    pub catalogue: PathBuf,
}

impl ResourceTypesRefreshArgs {
    pub async fn invoke(&self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let cancellation = executor.cancellation();
        Self::validate_catalogue_destination(&self.catalogue)?;
        cancellation.bail_if_cancelled()?;
        // App disables host authentication even when other workspace packages
        // enable its auth feature. Only this explicit network command resolves
        // the already-validated credential selection.
        let tenant_id = self.tenant.resolve(cancellation).await?;
        cancellation.bail_if_cancelled()?;
        let auth =
            AuthContext::resolve(self.auth_source.into())?.bind_to_azure_tenant(tenant_id)?;
        let executor = auth.auth_context.bind_executor(executor)?;
        let request = fetch_all_resource_providers(&auth, &executor);
        let catalogue = ResourceTypeCatalogue::from_providers(
            request.with_invalidation(true).await?,
            cancellation,
        )?;
        // Publishing happens only after every page and the complete vocabulary
        // have passed validation. A failed refresh preserves the existing file.
        Ok(CliOutput::facet(
            catalogue.save(&self.catalogue, cancellation)?,
        ))
    }

    fn validate_catalogue_destination(path: &Path) -> Result<()> {
        ensure!(
            path.to_str().is_some(),
            "catalogue output path must be representable as UTF-8 for command reports"
        );
        ensure!(
            path.file_name().is_some(),
            "catalogue output must name a file"
        );
        let parent = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        ensure!(
            fs::metadata(parent)
                .wrap_err_with(|| format!("examining catalogue directory {}", parent.display()))?
                .is_dir(),
            "catalogue parent must be an existing directory"
        );
        match fs::metadata(path) {
            Ok(metadata) => ensure!(
                metadata.is_file(),
                "catalogue output must name a regular file"
            ),
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(error) => return Err(error).wrap_err("examining catalogue destination"),
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::ResourceTypesRefreshArgs;
    use eyre::Result;
    use std::fs;

    #[test]
    fn invalid_output_destinations_are_rejected_before_authentication() -> Result<()> {
        let directory = tempfile::tempdir()?;
        assert!(
            ResourceTypesRefreshArgs::validate_catalogue_destination(directory.path()).is_err()
        );
        assert!(
            ResourceTypesRefreshArgs::validate_catalogue_destination(
                &directory.path().join("missing/catalogue.json")
            )
            .is_err()
        );
        let destination = directory.path().join("catalogue.json");
        assert!(ResourceTypesRefreshArgs::validate_catalogue_destination(&destination).is_ok());
        fs::write(&destination, "retained catalogue")?;
        assert!(ResourceTypesRefreshArgs::validate_catalogue_destination(&destination).is_ok());
        assert_eq!(fs::read_to_string(destination)?, "retained catalogue");
        Ok(())
    }
}
