use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use std::path::PathBuf;

/// Run the arena-backed reflected object explorer.
///
/// Engine ownership, lazy query execution, and bounded Ratatui state live in
/// their cohesive modules; this file is intentionally only public composition.
pub async fn ui_main(
    excalidraw: Option<PathBuf>,
    executor: &CloudTerrastodonExecutor,
) -> Result<()> {
    crate::object_browser::run_object_browser(excalidraw, executor).await
}
