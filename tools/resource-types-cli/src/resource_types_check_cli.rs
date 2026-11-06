use crate::ResourceTypeCatalogue;
use cloud_terrastodon_app::CancellationScope;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_app::Result;
use cloud_terrastodon_app::facet;
use cloud_terrastodon_app::figue;
use facet::Facet;
use std::path::PathBuf;

/// Compare generated source with a local catalogue.
#[derive(Facet)]
pub struct ResourceTypesCheckArgs {
    /// Local JSON array of full resource-type names, preserving service spelling.
    /// Defaults to catalogue.json beside the tool's Cargo.toml.
    #[facet(figue::named, default = ResourceTypeCatalogue::default_path())]
    pub catalogue: PathBuf,
    /// Existing Rust source file to check. A mismatch returns an error.
    /// Defaults to crates/azure_resource_types/src/resource_types.rs in the build checkout.
    #[facet(figue::named, default = ResourceTypeCatalogue::default_output_path())]
    pub output: PathBuf,
}

impl ResourceTypesCheckArgs {
    pub fn invoke(&self, cancellation: &CancellationScope) -> Result<CliOutput> {
        let catalogue = ResourceTypeCatalogue::read(&self.catalogue)?;
        cancellation.bail_if_cancelled()?;
        Ok(CliOutput::facet(
            catalogue.check(&self.output, cancellation)?,
        ))
    }
}
