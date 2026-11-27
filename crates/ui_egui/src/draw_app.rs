use crate::app::MyApp;
use crate::azure_tags_inventory_ui::azure_tags_inventory_ui;
use crate::entra_directory_ui::show_entra_directory;
use crate::file_drag_and_drop::ui_file_drag_and_drop;
use crate::portal_about::show_portal_about;
use crate::portal_auditing::show_portal_auditing;
use crate::portal_azure_devops_organization::show_azure_devops_organization;
use crate::portal_azure_devops_organization_view::PortalAzureDevOpsOrganizationView;
use crate::portal_azure_devops_project::show_azure_devops_project;
use crate::portal_home::portal_home;
use crate::portal_page::PortalPage;
use crate::portal_pim::show_portal_pim;
use crate::portal_resource_scope::PortalResourceScope;
use crate::portal_tab_target::PortalTabTarget;
use crate::resource_output_ui::resource_output_ui;
use crate::scope_detail::scope_detail_ui;
use crate::storage_explorer_ui::storage_container_ui;
use crate::windows::azure_devops_organizations_window::azure_devops_organizations_ui;
use crate::windows::dir_window::local_path_ui;
use crate::windows::management_groups_window::management_groups_ui;
use crate::windows::resource_groups_window::resource_groups_ui;
use crate::windows::resources_window::resources_of_type_ui;
use crate::windows::resources_window::resources_ui;
use crate::windows::selected_items_window::selected_items_ui;
use crate::windows::starting_points_window::starting_points_ui;
use crate::windows::work_dirs_window::DNDResponse;
use crate::windows::work_dirs_window::work_dirs_ui;
use eframe::egui::CentralPanel;
use eframe::egui::Color32;
use eframe::egui::Frame;
use eframe::egui::Margin;
use eframe::egui::Panel;
use eframe::egui::RichText;
use eframe::egui::Ui;

impl MyApp {
    pub fn draw_app(&mut self, ui: &mut Ui) {
        self.handle_history_input(ui);
        self.handle_portal_tab_shortcuts(ui);
        ui_file_drag_and_drop(self, ui.ctx());
        Panel::top("portal_browser_toolbar")
            .frame(
                Frame::new()
                    .fill(Color32::from_rgb(245, 248, 251))
                    .inner_margin(Margin::symmetric(12, 4)),
            )
            .show(ui, |ui| {
                let target = self.tabs.active().clone();
                self.show_portal_browser_toolbar(ui, &target);
            });
        let split_rails = self.tabs.is_split();
        if !split_rails {
            self.portal_tabs_bar(ui, self.tabs.active_pane());
        }
        self.portal_navigation(ui);
        CentralPanel::default()
            .frame(Frame::new().fill(Color32::WHITE))
            .show(ui, |ui| self.show_portal_workspace(ui, split_rails));
        self.show_portal_tab_drag_preview(ui);
        self.show_toasts(ui);
    }

    pub(crate) fn show_portal_tab(&mut self, ui: &mut Ui, target: PortalTabTarget) {
        let tab = self.tabs.active_id();
        if self.show_portal_visit_breadcrumbs(ui) {
            return;
        }
        self.load_portal_hierarchy_if_needed(&target);
        let standard_heading = !target.page().is_entra()
            && !target.page().is_about_section()
            && !target.page().is_audit_section()
            && !target.page().is_pim_section()
            && !matches!(
                target,
                PortalTabTarget::Page(PortalPage::Home)
                    | PortalTabTarget::ResourceGroup(_)
                    | PortalTabTarget::Subscription(_)
                    | PortalTabTarget::ScopeView { .. }
                    | PortalTabTarget::ManagementGroup(_)
                    | PortalTabTarget::StorageAccount(_)
                    | PortalTabTarget::StorageContainer(_)
                    | PortalTabTarget::Resource(_)
                    | PortalTabTarget::AzureDevOpsOrganization { .. }
                    | PortalTabTarget::AzureDevOpsProject { .. }
            );
        // Home places its account control beside the services heading.
        let show_picker = target.page() != PortalPage::Home && show_account_in_tab_header(&target);
        let navigated = if show_picker && !standard_heading {
            self.account_page_header(ui, |app, ui| app.show_portal_breadcrumbs(ui, &target))
        } else {
            self.show_portal_breadcrumbs(ui, &target)
        };
        if navigated || self.tabs.active_id() != tab {
            return;
        }
        if standard_heading {
            let title = |app: &mut MyApp, ui: &mut Ui| {
                let title = app.portal_tab_title(&target);
                ui.label(RichText::new(title.as_ref()).strong().size(22.0));
            };
            if show_picker {
                self.account_page_header(ui, title);
            } else {
                title(self, ui);
            }
            ui.add_space(20.0);
        }
        if self.tabs.active_id() != tab {
            return;
        }
        match target {
            PortalTabTarget::Page(
                PortalPage::Entra
                | PortalPage::EntraUsers
                | PortalPage::EntraGroups
                | PortalPage::EntraAppRegistrations
                | PortalPage::EntraMyUser,
            )
            | PortalTabTarget::EntraUser(_)
            | PortalTabTarget::EntraUserSection { .. }
            | PortalTabTarget::EntraGroup(_)
            | PortalTabTarget::EntraGroupSection { .. }
            | PortalTabTarget::EntraAppRegistration(_) => show_entra_directory(self, ui, &target),
            PortalTabTarget::ResourceGroup(id) => {
                scope_detail_ui(self, ui, &PortalResourceScope::ResourceGroup(id), None)
            }
            PortalTabTarget::Subscription(id) => {
                scope_detail_ui(self, ui, &PortalResourceScope::Subscription(id), None)
            }
            PortalTabTarget::ScopeView { scope, view } => {
                scope_detail_ui(self, ui, &scope, Some(view))
            }
            PortalTabTarget::ResourcesOfType(kind) => resources_of_type_ui(self, ui, &kind),
            PortalTabTarget::ManagementGroup(id) => {
                scope_detail_ui(self, ui, &PortalResourceScope::ManagementGroup(id), None)
            }
            PortalTabTarget::StorageAccount(id) => {
                scope_detail_ui(self, ui, &PortalResourceScope::StorageAccount(id), None)
            }
            PortalTabTarget::StorageContainer(id) => {
                Frame::NONE
                    .inner_margin(Margin::symmetric(16, 8))
                    .show(ui, |ui| storage_container_ui(self, ui, &id));
            }
            PortalTabTarget::Resource(id) => {
                scope_detail_ui(self, ui, &PortalResourceScope::Resource(id), None)
            }
            PortalTabTarget::ResourceOutput { resource, format } => {
                resource_output_ui(self, ui, &resource, format)
            }
            PortalTabTarget::LocalPath(path) => local_path_ui(self, ui, &path),
            PortalTabTarget::Page(PortalPage::Home) => portal_home(self, ui),
            PortalTabTarget::Page(PortalPage::History) => self.show_portal_history(ui),
            PortalTabTarget::Page(
                page
                @ (PortalPage::Pim | PortalPage::PimEntraRoles | PortalPage::PimAzureResources),
            ) => show_portal_pim(self, ui, page),
            PortalTabTarget::Page(
                PortalPage::About
                | PortalPage::Logs
                | PortalPage::BackgroundWork
                | PortalPage::Accounts
                | PortalPage::AzureTenants,
            )
            | PortalTabTarget::AzureTenant(_) => show_portal_about(self, ui, &target),
            PortalTabTarget::Page(
                page @ (PortalPage::Auditing
                | PortalPage::AzureDevOpsAudit
                | PortalPage::StorageAccountAudit),
            ) => show_portal_auditing(self, ui, page),
            PortalTabTarget::Page(
                PortalPage::Resources
                | PortalPage::Subscriptions
                | PortalPage::StorageAccounts
                | PortalPage::ContainerRegistries
                | PortalPage::KeyVaults,
            ) => resources_ui(self, ui),
            PortalTabTarget::Page(PortalPage::ResourceGroups) => resource_groups_ui(self, ui),
            PortalTabTarget::Page(PortalPage::Tags) => azure_tags_inventory_ui(self, ui),
            PortalTabTarget::Page(PortalPage::ManagementGroups) => management_groups_ui(self, ui),
            PortalTabTarget::Page(PortalPage::AzureDevOpsOrganizations) => {
                azure_devops_organizations_ui(self, ui)
            }
            PortalTabTarget::AzureDevOpsOrganization { organization, view } => {
                show_azure_devops_organization(self, ui, &organization, view)
            }
            PortalTabTarget::AzureDevOpsProject {
                organization,
                project,
                view,
            } => show_azure_devops_project(self, ui, &organization, project, view),
            PortalTabTarget::Page(PortalPage::StartingPoints) => starting_points_ui(self, ui),
            PortalTabTarget::Page(PortalPage::SelectedItems) => selected_items_ui(self, ui),
            PortalTabTarget::Page(PortalPage::WorkDirs) => {
                let mut response = None;
                work_dirs_ui(self, ui, &mut response);
                if let Some(DNDResponse {
                    source_index,
                    destination_index,
                }) = response
                {
                    if let Some(item) = self.work_dirs_config.work_dirs.remove_index(source_index) {
                        let destination_index = destination_index
                            .saturating_sub(usize::from(source_index < destination_index))
                            .min(self.work_dirs_config.work_dirs.len());
                        self.work_dirs_config
                            .work_dirs
                            .shift_insert(destination_index, item);
                    } else {
                        tracing::warn!(source_index, "Dragged work directory no longer exists");
                    }
                }
            }
            PortalTabTarget::Page(PortalPage::FileDrop) => {
                ui.label("Drop Terraform files or directories into the window to open them.");
            }
        }
    }
}

pub(crate) fn show_account_in_tab_header(target: &PortalTabTarget) -> bool {
    match target {
        PortalTabTarget::AzureTenant(_) | PortalTabTarget::AzureDevOpsProject { .. } => true,
        PortalTabTarget::AzureDevOpsOrganization { view, .. } => {
            *view != PortalAzureDevOpsOrganizationView::Overview
        }
        target => {
            let page = target.page();
            page.is_entra()
                || page.is_audit_section()
                || page.is_pim_section()
                || matches!(
                    page,
                    PortalPage::Home
                        | PortalPage::Resources
                        | PortalPage::Subscriptions
                        | PortalPage::ResourceGroups
                        | PortalPage::ManagementGroups
                        | PortalPage::StorageAccounts
                        | PortalPage::ContainerRegistries
                        | PortalPage::KeyVaults
                        | PortalPage::Tags
                        | PortalPage::StartingPoints
                        | PortalPage::SelectedItems
                        | PortalPage::History
                )
        }
    }
}
