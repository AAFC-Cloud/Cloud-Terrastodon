use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
pub mod app;
pub mod autosave_info;
mod azure_account_action;
mod azure_account_confirmation;
mod azure_account_details;
mod azure_account_management;
mod azure_account_manager;
mod azure_account_manager_page;
mod azure_account_operation;
mod azure_account_operation_event;
pub mod azure_account_picker;
mod azure_account_picker_action;
mod azure_account_registration;
mod azure_account_registration_form;
mod azure_tenant_management_action;
mod azure_tenant_management_target;
mod azure_tenant_management_update;
mod azure_tenant_manager;
mod azure_tracked_tenant_catalogue;
#[cfg(windows)]
mod bundled_window_icon;
pub mod draw_app;
pub mod file_drag_and_drop;
mod gui_work_cache;
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

use crate::gui_work_cache::GuiWorkCache;
use cloud_terrastodon_auth::CloudTerrastodonAccountStore;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_cache::WorkCacheActivity;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use std::sync::Arc;
use teamy_tracing::capture::LogBuffer;
use tracing::info;

/// Run the desktop UI with the supplied local account catalogue. The window owns
/// a dedicated credential provider so an explicit reload can adopt new sessions.
/// Cache access uses the caller's executor owner.
pub async fn egui_main(
    app_info: String,
    azure_devops_organization: Option<AzureDevOpsOrganizationUrl>,
    authentication: eyre::Result<Option<AzureBearerAuthenticationSelection>>,
    account_store: CloudTerrastodonAccountStore,
    log_capture: LogBuffer,
    executor: &CloudTerrastodonExecutor,
) -> eyre::Result<()> {
    info!("Hello from egui!");
    run_app::run_app(
        app_info,
        azure_devops_organization,
        authentication,
        account_store,
        log_capture,
        executor,
    )
    .await?;
    Ok(())
}

/// Run the standalone desktop UI with cache ownership scoped to active jobs.
///
/// The supplied executor anchors cancellation and authentication. Its cache
/// must remain unacquired when it points at the same root as `activity`.
pub async fn egui_main_with_shared_cache(
    app_info: String,
    azure_devops_organization: Option<AzureDevOpsOrganizationUrl>,
    authentication: eyre::Result<Option<AzureBearerAuthenticationSelection>>,
    account_store: CloudTerrastodonAccountStore,
    log_capture: LogBuffer,
    executor: &CloudTerrastodonExecutor,
    activity: WorkCacheActivity,
) -> eyre::Result<()> {
    info!("Hello from egui!");
    run_app::run_app_with_work_cache(
        app_info,
        azure_devops_organization,
        authentication,
        account_store,
        log_capture,
        executor,
        GuiWorkCache::ActivityScoped(Arc::new(activity)),
    )
    .await
}
