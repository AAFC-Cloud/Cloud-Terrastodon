use eyre::Result;

/// Run the arena-backed reflected object explorer.
///
/// Engine ownership, lazy query execution, and bounded Ratatui state live in
/// their cohesive modules; this file is intentionally only public composition.
pub async fn ui_main() -> Result<()> {
    crate::object_browser::run_object_browser().await
}
