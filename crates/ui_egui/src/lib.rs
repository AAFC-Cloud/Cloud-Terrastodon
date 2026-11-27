use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
pub mod app;
pub mod autosave_info;
mod azure_account_action;
mod azure_account_confirmation;
mod azure_account_details;
mod azure_account_inventory_row;
mod azure_account_management;
mod azure_account_manager;
mod azure_account_manager_page;
mod azure_account_operation;
mod azure_account_operation_event;
mod azure_account_operation_origin;
pub mod azure_account_picker;
mod azure_account_picker_action;
mod azure_account_profile;
mod azure_account_profiles;
mod azure_account_registration;
mod azure_account_registration_form;
mod azure_account_sign_in_method;
mod azure_devops_organization_picker;
mod azure_devops_project_data;
mod azure_devops_project_data_ui;
mod azure_devops_project_inventory;
mod azure_tag_inventory;
mod azure_tag_inventory_entry;
mod azure_tag_inventory_row;
mod azure_tags_inventory_ui;
mod azure_tenant_management_action;
mod azure_tenant_management_target;
mod azure_tenant_management_update;
mod azure_tenant_manager;
mod azure_tracked_tenant_catalogue;
mod azure_tracked_tenant_row;
#[cfg(windows)]
mod bundled_window_icon;
mod capture_gui_failure;
pub mod draw_app;
mod entra_application_credential_metadata;
mod entra_application_credential_row;
mod entra_application_credential_status;
mod entra_application_registration_row;
mod entra_application_registrations_ui;
mod entra_directory;
mod entra_directory_inventory;
mod entra_directory_ui;
mod entra_devops_permissions;
mod entra_group_data;
mod entra_group_directory_object_row;
pub mod entra_group_section;
mod entra_group_sections_ui;
mod entra_role_assignment_row;
mod entra_user_application_assignment_row;
mod entra_user_application_data;
mod entra_user_application_sections_ui;
mod entra_user_groups;
mod entra_user_groups_ui;
mod entra_user_permissions;
pub mod entra_user_section;
pub mod entra_user_subject;
pub mod file_drag_and_drop;
mod gui_work_cache;
pub mod icons;
mod inventory_table;
mod local_path_preview;
mod management_group_hierarchy;
mod management_group_inventory_row;
mod management_group_overview;
mod management_group_tree;
mod management_group_tree_row;
mod network_security_group_rule_rows;
mod network_security_group_rules;
mod network_security_rule_row;
mod pim_activation;
mod pim_activation_assignment;
mod pim_assignment_status;
mod pim_entra_role_assignment;
mod pim_role_inventory;
mod pim_role_row;
mod portal_about;
mod portal_address;
mod portal_address_format;
mod portal_audit_output;
mod portal_auditing;
mod portal_azure_audit;
mod portal_azure_content;
mod portal_azure_devops_audit;
mod portal_azure_devops_organization;
pub mod portal_azure_devops_organization_view;
mod portal_azure_devops_project;
pub mod portal_azure_devops_project_view;
mod portal_background_work;
mod portal_breadcrumbs;
mod portal_browser_control;
mod portal_browser_toolbar;
mod portal_filter_chip;
mod portal_hierarchy;
mod portal_history;
mod portal_history_entry;
mod portal_history_ui;
pub mod portal_home;
mod portal_log_exclusion;
mod portal_log_toast;
mod portal_log_ui;
pub mod portal_navigation;
mod portal_omnibox;
pub mod portal_page;
mod portal_pane;
mod portal_pane_id;
mod portal_pane_layout;
mod portal_pim;
mod portal_refresh;
pub mod portal_resource_output_target;
pub mod portal_resource_scope;
pub mod portal_scope_view;
mod portal_search;
mod portal_section_tabs;
mod portal_split;
mod portal_split_handle;
mod portal_split_side;
mod portal_storage_audit_findings;
mod portal_storage_audit_format;
mod portal_storage_audit_row;
mod portal_tab;
mod portal_tab_action;
mod portal_tab_drag;
mod portal_tab_group;
mod portal_tab_group_id;
mod portal_tab_group_ui;
mod portal_tab_id;
mod portal_tab_menu;
mod portal_tab_open;
pub mod portal_tab_target;
mod portal_tabs;
mod portal_tabs_bar;
mod portal_toasts;
pub mod portal_typography;
mod portal_workspace;
mod reflected_table;
mod reflected_table_column;
mod reflected_table_group;
mod reflected_table_group_heading;
mod reflected_table_rows;
mod reflected_table_sort;
#[cfg(test)]
mod reflected_table_virtualization_tests;
mod reflected_value;
mod resource_azure_essentials;
mod resource_detail_header;
mod resource_essentials;
mod resource_essentials_format;
mod resource_group_browser;
mod resource_group_filters;
mod resource_group_list_command;
mod resource_inventory;
mod resource_inventory_grouping;
mod resource_inventory_row;
mod resource_output;
mod resource_output_ui;
mod resource_snapshot;
pub mod run_app;
mod scope_activity;
mod scope_activity_filters;
mod scope_activity_log;
mod scope_activity_period;
mod scope_activity_results;
mod scope_deny_assignment_row;
mod scope_detail;
mod scope_iam;
mod scope_iam_ancestry;
mod scope_iam_browser;
mod scope_iam_comparison;
mod scope_iam_denies;
mod scope_iam_inventory;
mod scope_iam_principals;
mod scope_iam_section;
mod scope_iam_snapshot;
mod scope_iam_thumbnails;
mod scope_navigation;
mod scope_role_assignment_filters;
mod scope_role_assignment_grouping;
mod scope_role_assignment_principal_link;
mod scope_role_assignment_row;
mod scope_role_assignment_scope_filter;
mod scope_role_assignments_view;
mod scope_role_definition_row;
mod scope_tag_draft_row;
mod scope_tags_browser;
mod scope_tags_editor;
mod scope_tags_ui;
mod sidebar_expansion_toggle;
mod storage_account_browser;
mod storage_account_inspection;
mod storage_account_show_command;
mod storage_explorer;
mod storage_explorer_container_row;
mod storage_explorer_ui;
mod storage_networking;
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
