use super::excalidraw_interactive_session::ExcalidrawInteractiveSession;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_command::CancellationScope;
use eyre::Result;
use std::path::PathBuf;

/// A local file session: no credentials or cloud requests are used.
#[derive(Debug, facet::Facet)]
pub struct ExcalidrawInteractiveArgs {
    /// Drawing to watch. A missing file is initialized with example controls.
    #[facet(figue::positional)]
    pub path: PathBuf,
}

impl ExcalidrawInteractiveArgs {
    pub async fn invoke(self, cancellation: &CancellationScope) -> Result<CliOutput> {
        ExcalidrawInteractiveSession::open(self.path, cancellation)
            .await?
            .run(cancellation)
            .await?;
        Ok(CliOutput::none())
    }
}

cloud_terrastodon_registry::register_thing!(ExcalidrawInteractiveArgs);
