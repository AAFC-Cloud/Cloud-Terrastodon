use crate::version::full_version;
use clap::Args;
use cloud_terrastodon_azure::prelude::get_default_tenant_id;
use cloud_terrastodon_ui_egui::egui_main;

/// Launch the egui-based graphical interface.
#[derive(Args, Debug, Clone, Default)]
pub struct EguiArgs;

impl EguiArgs {
    pub async fn invoke(self) -> eyre::Result<()> {
        let tenant_id = get_default_tenant_id().await?;
        egui_main(full_version().to_string(), tenant_id).await?;
        Ok(())
    }
}
