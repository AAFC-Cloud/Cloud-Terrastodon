use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_ui_ratatui::ui_main;
use eyre::Result;

/// Launch the Ratatui-based interface.
#[derive(facet::Facet, Debug, Clone, Default)]
pub struct RatatuiArgs;

impl RatatuiArgs {
    pub async fn invoke(self, cancellation: &CancellationScope) -> Result<()> {
        ui_main(cancellation).await?;
        Ok(())
    }
}
