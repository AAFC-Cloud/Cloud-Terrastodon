use crate::app::MyApp;
use crate::azure_account_picker_action::AzureAccountPickerAction;
use crate::portal_page::PortalPage;
use crate::portal_resource_output_target::PortalResourceOutputTarget;
use crate::portal_resource_scope::PortalResourceScope;
use crate::portal_scope_view::PortalScopeView;
use crate::portal_tab_target::PortalTabTarget;
use crate::scope_activity::load_scope_activity;
use crate::scope_iam::load_scope_iam;
use crate::storage_account_browser::load_storage_account;
use crate::windows::dir_window::load_local_path;
use crate::windows::management_groups_window::load_management_groups;
use crate::windows::resources_window::load_resources;
use crate::workers::load_azure_devops_projects::load_azure_devops_projects_with_invalidation;
use crate::workers::load_resource_groups::load_resource_groups;
use crate::workers::load_subscriptions::load_subscriptions_with_invalidation;
use cloud_terrastodon_command::app_work::Loadable;
use eframe::egui::Ui;

impl MyApp {
    pub(crate) fn can_refresh_portal_view(&self, target: &PortalTabTarget) -> bool {
        match target {
            PortalTabTarget::Page(
                PortalPage::Home
                | PortalPage::Resources
                | PortalPage::Subscriptions
                | PortalPage::StorageAccounts
                | PortalPage::ContainerRegistries
                | PortalPage::KeyVaults,
            )
            | PortalTabTarget::ResourcesOfType(_) => {
                self.ready_azure_authentication().is_some()
                    && !matches!(self.resources, Loadable::Loading { .. })
            }
            PortalTabTarget::Page(PortalPage::ResourceGroups) => {
                self.ready_azure_authentication().is_some()
                    && !matches!(self.resource_groups, Loadable::Loading { .. })
            }
            PortalTabTarget::Page(PortalPage::ManagementGroups) => {
                self.ready_azure_authentication().is_some()
                    && !matches!(self.management_groups, Loadable::Loading { .. })
            }
            PortalTabTarget::ManagementGroup(_) => {
                self.ready_azure_authentication().is_some()
                    && !matches!(self.management_groups, Loadable::Loading { .. })
                    && !matches!(self.subscriptions, Loadable::Loading { .. })
            }
            PortalTabTarget::ResourceGroup(_) => {
                self.ready_azure_authentication().is_some()
                    && !matches!(self.resource_groups, Loadable::Loading { .. })
                    && !matches!(self.resources, Loadable::Loading { .. })
            }
            PortalTabTarget::Subscription(_) => {
                self.ready_azure_authentication().is_some()
                    && !matches!(self.subscriptions, Loadable::Loading { .. })
                    && !matches!(self.resources, Loadable::Loading { .. })
            }
            PortalTabTarget::ScopeView { scope, view } => {
                self.ready_azure_authentication().is_some()
                    && match view {
                        PortalScopeView::ActivityLog => !matches!(
                            self.scope_activity.get(scope),
                            Some(Loadable::Loading { .. })
                        ),
                        PortalScopeView::AccessControl => !matches!(
                            self.scope_iam.get(&scope.arm_scope()),
                            Some(Loadable::Loading { .. })
                        ),
                    }
            }
            PortalTabTarget::ResourceOutput {
                resource: PortalResourceOutputTarget::Scope(scope),
                ..
            } => {
                self.ready_azure_authentication().is_some()
                    && match scope {
                        PortalResourceScope::ResourceGroup(_) => {
                            !matches!(self.resource_groups, Loadable::Loading { .. })
                        }
                        PortalResourceScope::Subscription(_) => {
                            !matches!(self.subscriptions, Loadable::Loading { .. })
                        }
                    }
            }
            PortalTabTarget::StorageAccount(id)
            | PortalTabTarget::ResourceOutput {
                resource: PortalResourceOutputTarget::StorageAccount(id),
                ..
            } => {
                self.ready_azure_authentication().is_some()
                    && !self
                        .storage_account_browser
                        .entries
                        .get(id)
                        .is_some_and(|entry| matches!(entry.response, Loadable::Loading { .. }))
            }
            PortalTabTarget::Page(PortalPage::AzureDevOpsProjects) => {
                self.ready_azure_devops_authentication().is_some()
                    && !matches!(self.azure_devops_projects, Loadable::Loading { .. })
            }
            PortalTabTarget::Page(PortalPage::Accounts | PortalPage::AzureDevOpsOrganizations) => {
                self.account_operation.is_none()
            }
            PortalTabTarget::Page(PortalPage::AzureTenants) => {
                self.account_operation.is_none() && self.tenant_manager.can_reload_catalogue()
            }
            PortalTabTarget::LocalPath(path) => !matches!(
                self.local_path_previews.get(path),
                Some(Loadable::Loading { .. })
            ),
            PortalTabTarget::Page(
                PortalPage::StartingPoints
                | PortalPage::SelectedItems
                | PortalPage::WorkDirs
                | PortalPage::FileDrop
                | PortalPage::Logs,
            ) => false,
        }
    }

    pub(crate) fn refresh_portal_view(&mut self, ui: &mut Ui, target: &PortalTabTarget) {
        if !self.can_refresh_portal_view(target) {
            return;
        }
        match target {
            PortalTabTarget::Page(
                PortalPage::Home
                | PortalPage::Resources
                | PortalPage::Subscriptions
                | PortalPage::StorageAccounts
                | PortalPage::ContainerRegistries
                | PortalPage::KeyVaults,
            )
            | PortalTabTarget::ResourcesOfType(_) => load_resources(self, true),
            PortalTabTarget::Page(PortalPage::ResourceGroups) => {
                load_resource_groups(self, true);
            }
            PortalTabTarget::Page(PortalPage::ManagementGroups) => {
                load_management_groups(self, true)
            }
            PortalTabTarget::ManagementGroup(_) => {
                load_management_groups(self, true);
                load_subscriptions_with_invalidation(self, true);
            }
            PortalTabTarget::ResourceGroup(_) => {
                load_resource_groups(self, true);
                load_resources(self, true);
            }
            PortalTabTarget::Subscription(_) => {
                load_subscriptions_with_invalidation(self, true);
                load_resources(self, true);
            }
            PortalTabTarget::ScopeView { scope, view } => match view {
                PortalScopeView::ActivityLog => load_scope_activity(self, scope, true),
                PortalScopeView::AccessControl => load_scope_iam(self, &scope.arm_scope(), true),
            },
            PortalTabTarget::ResourceOutput {
                resource: PortalResourceOutputTarget::Scope(scope),
                ..
            } => match scope {
                PortalResourceScope::ResourceGroup(_) => load_resource_groups(self, true),
                PortalResourceScope::Subscription(_) => {
                    load_subscriptions_with_invalidation(self, true)
                }
            },
            PortalTabTarget::StorageAccount(id)
            | PortalTabTarget::ResourceOutput {
                resource: PortalResourceOutputTarget::StorageAccount(id),
                ..
            } => {
                load_storage_account(self, id, true);
            }
            PortalTabTarget::Page(PortalPage::AzureDevOpsProjects) => {
                load_azure_devops_projects_with_invalidation(self, true);
            }
            PortalTabTarget::Page(PortalPage::Accounts | PortalPage::AzureDevOpsOrganizations) => {
                self.handle_account_picker_action(AzureAccountPickerAction::Reload, ui.ctx());
            }
            PortalTabTarget::Page(PortalPage::AzureTenants) => {
                self.tenant_manager.reload_catalogue(
                    &self.work_tracker,
                    &self.executor,
                    &self.work_cache,
                    ui.ctx(),
                )
            }
            PortalTabTarget::LocalPath(path) => load_local_path(self, path),
            PortalTabTarget::Page(
                PortalPage::StartingPoints
                | PortalPage::SelectedItems
                | PortalPage::WorkDirs
                | PortalPage::FileDrop
                | PortalPage::Logs,
            ) => {}
        }
    }
}
