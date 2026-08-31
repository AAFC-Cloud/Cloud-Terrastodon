pub mod app;
pub mod app_message;
pub mod autosave_info;
pub mod draw_app;
pub mod file_drag_and_drop;
pub mod icons;
pub mod loadable;
pub mod loadable_work;
pub mod run_app;
pub mod state_mutator;
pub mod tiles;
pub mod widgets;
pub mod windows;
pub mod work;
pub mod work_tracker;
pub mod workers;

use cloud_terrastodon_azure::AzureTenantId;
use cloud_terrastodon_credentials::AuthContext;
use tracing::info;

pub async fn egui_main(
    app_info: String,
    tenant_id: AzureTenantId,
    auth_context: AuthContext,
) -> eyre::Result<()> {
    info!("Hello from egui!");
    run_app::run_app(app_info, tenant_id, auth_context).await?;
    Ok(())
}
