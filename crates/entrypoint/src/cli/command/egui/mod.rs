use clap::Args;
use cloud_terrastodon_ui_egui::egui_main;
use eyre::Result;

/// Launch the egui-based graphical interface.
#[derive(Args, Debug, Clone, Default)]
pub struct EguiArgs;

impl EguiArgs {
    pub async fn invoke(self) -> Result<()> {
        egui_main().await?;
        Ok(())
    }
}
