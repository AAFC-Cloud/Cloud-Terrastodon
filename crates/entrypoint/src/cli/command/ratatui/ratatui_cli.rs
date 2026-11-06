use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_ui_ratatui::ui_main;
use eyre::Result;
use facet::Facet;
use std::path::PathBuf;

/// Explore reflected objects and export their diagrams to Excalidraw.
#[derive(Facet, Debug, Clone, Default)]
pub struct RatatuiArgs {
    /// Disposable drawing replaced by Ctrl-O (focused slot) or Ctrl-U (current tab).
    /// When omitted, the first export creates a unique file in the working directory.
    #[facet(figue::named)]
    pub excalidraw: Option<PathBuf>,
}

impl RatatuiArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<()> {
        ui_main(self.excalidraw, executor).await
    }
}
