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
mod azure_account_profile;
mod azure_account_profiles;
mod azure_account_registration;
mod azure_account_registration_form;
mod azure_account_sign_in_method;
mod azure_devops_organization_picker;
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
mod inventory_table;
mod local_path_preview;
mod portal_address;
mod portal_breadcrumbs;
mod portal_browser_control;
mod portal_browser_toolbar;
pub mod portal_header;
mod portal_hierarchy;
pub mod portal_home;
pub mod portal_navigation;
pub mod portal_page;
mod portal_refresh;
pub mod portal_resource_scope;
pub mod portal_scope_view;
mod portal_tab;
pub mod portal_tab_target;
mod portal_tabs;
mod portal_tabs_bar;
pub mod portal_typography;
mod reflected_table;
mod reflected_table_column;
mod reflected_table_sort;
mod reflected_value;
mod resource_group_browser;
mod resource_group_filters;
mod resource_group_list_command;
mod resource_inventory_row;
pub mod run_app;
mod scope_activity;
mod scope_deny_assignment_row;
mod scope_detail;
mod scope_iam;
mod scope_iam_ancestry;
mod scope_iam_comparison;
mod scope_iam_section;
mod scope_iam_snapshot;
mod scope_navigation;
mod scope_role_assignment_row;
mod scope_role_definition_row;
mod sidebar_expansion_toggle;
mod storage_account_browser;
mod storage_account_inspection;
mod storage_account_output;
mod storage_account_show_command;
mod table_row_background;
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
