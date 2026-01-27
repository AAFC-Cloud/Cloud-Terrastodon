use crate::version::full_version;
use clap::Args;
use cloud_terrastodon_ui_egui::egui_main;

/// Launch the egui-based graphical interface.
#[derive(Args, Debug, Clone, Default)]
pub struct EguiArgs;

impl EguiArgs {
    pub async fn invoke(self) -> eyre::Result<()> {
        egui_main(full_version().to_string()).await?;
        Ok(())
    }
}
