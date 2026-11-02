use crate::ResourceTypeCatalogue;
use cloud_terrastodon_app::CancellationScope;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_app::Result;
use cloud_terrastodon_app::facet;
use cloud_terrastodon_app::figue;
use eyre::WrapErr;
use eyre::ensure;
use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;

/// Render a complete catalogue to the generated Rust source file.
#[derive(facet::Facet)]
pub struct ResourceTypesGenerateArgs {
    /// Local JSON array of full resource-type names, preserving service spelling.
    /// Defaults to catalogue.json beside the tool's Cargo.toml.
    #[facet(figue::named, default = ResourceTypeCatalogue::default_path())]
    pub catalogue: PathBuf,
    /// Destination Rust source file. Its parent directory must already exist.
    /// Defaults to crates/azure_resource_types/src/resource_types.rs in the build checkout.
    #[facet(figue::named, default = ResourceTypeCatalogue::default_output_path())]
    pub output: PathBuf,
}

impl ResourceTypesGenerateArgs {
    pub fn invoke(&self, cancellation: &CancellationScope) -> Result<CliOutput> {
        let catalogue = ResourceTypeCatalogue::read(&self.catalogue)?;
        cancellation.bail_if_cancelled()?;
        match fs::canonicalize(&self.output) {
            Ok(output) => ensure!(
                output != fs::canonicalize(&self.catalogue).wrap_err("resolving catalogue path")?,
                "generated output must not replace its input catalogue"
            ),
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(error) => return Err(error).wrap_err("resolving generated output path"),
        }
        Ok(CliOutput::facet(
            catalogue.generate(&self.output, cancellation)?,
        ))
    }
}
