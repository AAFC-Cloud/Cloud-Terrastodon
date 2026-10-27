use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_ui_ratatui::ui_main;
use eyre::Result;
use std::path::PathBuf;

/// Explore reflected objects and export their diagrams to Excalidraw.
#[derive(facet::Facet, Debug, Clone, Default)]
pub struct RatatuiArgs {
    /// Disposable drawing replaced by Ctrl-O (focused slot) or Ctrl-U (current tab).
    /// When omitted, the first export creates a unique file in the working directory.
    #[facet(figue::named)]
    pub excalidraw: Option<PathBuf>,
}

impl RatatuiArgs {
    pub async fn invoke(self, cancellation: &CancellationScope) -> Result<()> {
        ui_main(self.excalidraw, cancellation).await
    }
}
