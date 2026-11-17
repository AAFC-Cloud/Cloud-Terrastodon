use crate::IconCatalogue;
use crate::generation;
use cloud_terrastodon_app::CancellationScope;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_app::Result;
use cloud_terrastodon_app::figue;
use facet::Facet;
use std::path::PathBuf;

/// Verify asset integrity and generated output without network access or writes.
#[derive(Facet)]
pub struct IconographyCheckArgs {
    /// Cloud Terrastodon workspace root containing the three iconography crates.
    #[facet(figue::named, default = PathBuf::from("."), figue::label = "DIRECTORY")]
    pub workspace: PathBuf,
}

impl IconographyCheckArgs {
    pub fn invoke(self, cancellation: &CancellationScope) -> Result<CliOutput> {
        Ok(CliOutput::facet(generation::check(
            &self.workspace,
            &IconCatalogue::read(&self.workspace)?,
            cancellation,
        )?))
    }
}
