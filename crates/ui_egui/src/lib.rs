use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
pub mod app;
pub mod autosave_info;
pub mod azure_account_picker;
#[cfg(windows)]
mod bundled_window_icon;
pub mod draw_app;
pub mod file_drag_and_drop;
pub mod icons;
pub mod portal_header;
pub mod portal_home;
pub mod portal_navigation;
pub mod portal_page;
pub mod portal_typography;
pub mod run_app;
pub mod widgets;
pub mod windows;
pub mod workers;

use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use teamy_tracing::capture::LogBuffer;
use tracing::info;

pub async fn egui_main(
    app_info: String,
    azure_devops_organization: Option<AzureDevOpsOrganizationUrl>,
    authentication: AzureBearerAuthenticationSelection,
    log_capture: LogBuffer,
    executor: &CloudTerrastodonExecutor,
) -> eyre::Result<()> {
    info!("Hello from egui!");
    run_app::run_app(
        app_info,
        azure_devops_organization,
        authentication,
        log_capture,
        executor,
    )
    .await?;
    Ok(())
}
