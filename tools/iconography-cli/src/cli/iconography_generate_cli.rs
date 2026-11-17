use crate::IconCatalogue;
use crate::generation;
use cloud_terrastodon_app::CancellationScope;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_app::Result;
use cloud_terrastodon_app::figue;
use facet::Facet;
use std::path::PathBuf;

/// Regenerate the checked-in Rust and feature declarations using local inputs.
#[derive(Facet)]
pub struct IconographyGenerateArgs {
    /// Cloud Terrastodon workspace root containing the three iconography crates.
    #[facet(figue::named, default = PathBuf::from("."), figue::label = "DIRECTORY")]
    pub workspace: PathBuf,
}

impl IconographyGenerateArgs {
    pub fn invoke(self, cancellation: &CancellationScope) -> Result<CliOutput> {
        Ok(CliOutput::facet(generation::generate(
            &self.workspace,
            &IconCatalogue::read(&self.workspace)?,
            cancellation,
        )?))
    }
}
