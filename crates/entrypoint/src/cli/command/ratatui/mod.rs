use clap::Args;
use cloud_terrastodon_ui_ratatui::prelude::ui_main;
use eyre::Result;

/// Launch the Ratatui-based interface.
#[derive(Args, Debug, Clone, Default)]
pub struct RatatuiArgs;

impl RatatuiArgs {
    pub async fn invoke(self) -> Result<()> {
        ui_main().await?;
        Ok(())
    }
}
