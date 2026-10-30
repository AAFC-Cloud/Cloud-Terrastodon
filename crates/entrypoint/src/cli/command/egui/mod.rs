use crate::version::full_version;
use cloud_terrastodon_azure::get_default_tenant_id;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_credentials::AuthContext;
use cloud_terrastodon_ui_egui::egui_main;

/// Launch the egui-based graphical interface.
#[derive(facet::Facet, Debug, Clone, Default)]
pub struct EguiArgs;

impl EguiArgs {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        cancellation: &CancellationScope,
    ) -> eyre::Result<()> {
        let tenant_id = get_default_tenant_id(cancellation).await?;
        egui_main(
            full_version()?.to_string(),
            tenant_id,
            auth_context.clone(),
            cancellation,
        )
        .await?;
        Ok(())
    }
}
