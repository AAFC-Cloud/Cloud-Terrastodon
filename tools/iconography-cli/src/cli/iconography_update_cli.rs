use crate::IconFamily;
use crate::importer;
use cloud_terrastodon_app::CancellationScope;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_app::Result;
use cloud_terrastodon_app::figue;
use facet::Facet;
use std::path::PathBuf;

/// Select the public icon source to refresh in a Cloud Terrastodon worktree.
#[derive(Facet)]
pub struct IconographyUpdateArgs {
    /// Cloud Terrastodon workspace root containing the three iconography crates.
    #[facet(figue::named, default = PathBuf::from("."), figue::label = "DIRECTORY")]
    pub workspace: PathBuf,
    /// Public asset family: azure or azure-devops.
    #[facet(figue::named, figue::label = "SOURCE")]
    pub source: IconFamily,
}

impl IconographyUpdateArgs {
    pub async fn invoke(self, cancellation: &CancellationScope) -> Result<CliOutput> {
        Ok(CliOutput::facet(
            importer::update(&self.workspace, self.source, cancellation).await?,
        ))
    }
}
