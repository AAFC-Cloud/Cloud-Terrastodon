use crate::app::MyApp;
use crate::file_drag_and_drop::ui_file_drag_and_drop;
use crate::portal_home::portal_home;
use crate::portal_page::PortalPage;
use crate::portal_resource_scope::PortalResourceScope;
use crate::portal_tab_target::PortalTabTarget;
use crate::portal_typography;
use crate::scope_detail::scope_detail_ui;
use crate::windows::azure_devops_organizations_window::azure_devops_organizations_ui;
use crate::windows::azure_devops_projects_window::azure_devops_projects_ui;
use crate::windows::dir_window::local_path_ui;
use crate::windows::management_groups_window::management_group_details_ui;
use crate::windows::management_groups_window::management_groups_ui;
use crate::windows::resource_groups_window::resource_groups_ui;
use crate::windows::resources_window::resources_of_type_ui;
use crate::windows::resources_window::resources_ui;
use crate::windows::selected_items_window::selected_items_ui;
use crate::windows::starting_points_window::starting_points_ui;
use crate::windows::storage_account_output_window::storage_account_output_ui;
use crate::windows::storage_account_window::storage_account_ui;
use crate::windows::work_dirs_window::DNDResponse;
use crate::windows::work_dirs_window::work_dirs_ui;
use eframe::egui::Align2;
use eframe::egui::CentralPanel;
use eframe::egui::Color32;
use eframe::egui::Frame;
use eframe::egui::Margin;
use eframe::egui::Panel;
use eframe::egui::RichText;
use eframe::egui::ScrollArea;
use eframe::egui::Ui;
use eframe::egui::Window;

impl MyApp {
    pub fn draw_app(&mut self, ui: &mut Ui) {
        self.handle_mouse_history(ui);
        ui_file_drag_and_drop(self, ui.ctx());
        self.portal_header(ui);
        self.portal_navigation(ui);
        Panel::top("portal_browser_toolbar")
            .frame(
                Frame::new()
                    .fill(Color32::from_rgb(245, 248, 251))
                    .inner_margin(Margin::symmetric(12, 6)),
            )
            .show(ui, |ui| {
                let target = self.tabs.active().clone();
                self.show_portal_browser_toolbar(ui, &target);
            });
        self.portal_tabs_bar(ui);
        CentralPanel::default()
            .frame(Frame::new().fill(Color32::WHITE))
            .show(ui, |ui| {
                let target = self.tabs.active().clone();
                self.load_portal_hierarchy_if_needed(&target);
                if self.show_portal_breadcrumbs(ui, &target) {
                    return;
                }
                if !matches!(
                    target,
                    PortalTabTarget::Page(PortalPage::Home)
                        | PortalTabTarget::ResourceGroup(_)
                        | PortalTabTarget::Subscription(_)
                        | PortalTabTarget::ScopeView { .. }
                        | PortalTabTarget::ManagementGroup(_)
                        | PortalTabTarget::StorageAccount(_)
                ) {
                    let title = self.portal_tab_title(&target);
                    ui.label(RichText::new(title.as_ref()).strong().size(22.0));
                    ui.add_space(20.0);
                }
                match target {
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
                        management_group_details_ui(self, ui, &id)
                    }
                    PortalTabTarget::StorageAccount(id) => storage_account_ui(self, ui, &id),
                    PortalTabTarget::StorageAccountOutput { id, format } => {
                        storage_account_output_ui(self, ui, &id, format)
                    }
                    PortalTabTarget::LocalPath(path) => local_path_ui(self, ui, &path),
                    PortalTabTarget::Page(PortalPage::Home) => portal_home(self, ui),
                    PortalTabTarget::Page(PortalPage::Logs) => {
                        self.log_view.show(ui, &self.log_capture);
                        // UI refresh is CT policy; capture has no dependency on a window.
                        ui.ctx()
                            .request_repaint_after(std::time::Duration::from_millis(250));
                    }
                    PortalTabTarget::Page(
                        PortalPage::Resources
                        | PortalPage::Subscriptions
                        | PortalPage::StorageAccounts
                        | PortalPage::ContainerRegistries
                        | PortalPage::KeyVaults,
                    ) => resources_ui(self, ui),
                    PortalTabTarget::Page(PortalPage::ResourceGroups) => {
                        resource_groups_ui(self, ui)
                    }
                    PortalTabTarget::Page(PortalPage::ManagementGroups) => {
                        management_groups_ui(self, ui)
                    }
                    PortalTabTarget::Page(PortalPage::AzureDevOpsOrganizations) => {
                        azure_devops_organizations_ui(self, ui)
                    }
                    PortalTabTarget::Page(PortalPage::AzureDevOpsProjects) => {
                        azure_devops_projects_ui(self, ui)
                    }
                    PortalTabTarget::Page(PortalPage::Accounts) => self.show_accounts_page(ui),
                    PortalTabTarget::Page(PortalPage::AzureTenants) => {
                        self.show_azure_tenants_page(ui)
                    }
                    PortalTabTarget::Page(PortalPage::StartingPoints) => {
                        starting_points_ui(self, ui)
                    }
                    PortalTabTarget::Page(PortalPage::SelectedItems) => selected_items_ui(self, ui),
                    PortalTabTarget::Page(PortalPage::WorkDirs) => {
                        let mut response = None;
                        work_dirs_ui(self, ui, &mut response);
                        if let Some(DNDResponse {
                            source_index,
                            destination_index,
                        }) = response
                        {
                            if let Some(item) =
                                self.work_dirs_config.work_dirs.remove_index(source_index)
                            {
                                let destination_index = destination_index
                                    .saturating_sub(usize::from(source_index < destination_index))
                                    .min(self.work_dirs_config.work_dirs.len());
                                self.work_dirs_config
                                    .work_dirs
                                    .shift_insert(destination_index, item);
                            } else {
                                tracing::warn!(
                                    source_index,
                                    "Dragged work directory no longer exists"
                                );
                            }
                        }
                    }
                    PortalTabTarget::Page(PortalPage::FileDrop) => {
                        ui.label(
                            "Drop Terraform files or directories into the window to open them.",
                        );
                    }
                }
            });
        if self.about_open {
            Window::new("About Cloud Terrastodon")
                .resizable(false)
                .collapsible(false)
                .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
                .open(&mut self.about_open)
                .show(ui.ctx(), |ui| {
                    ui.heading("Cloud Terrastodon");
                    ui.add_space(10.0);
                    ui.label(&self.app_info);
                    ui.add_space(10.0);
                    ui.hyperlink_to("GitHub", "https://github.com/AAFC-Cloud/Cloud-Terrastodon");
                    ui.add_space(10.0);
                    ui.collapsing("Allo font license", |ui| {
                        ScrollArea::vertical().max_height(240.0).show(ui, |ui| {
                            ui.label(portal_typography::LICENSE);
                        });
                    });
                });
        }
        self.toasts.show(ui);
    }
}
