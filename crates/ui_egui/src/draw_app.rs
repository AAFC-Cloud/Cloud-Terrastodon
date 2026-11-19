use crate::app::MyApp;
use crate::file_drag_and_drop::file_drag_ui;
use crate::portal_home::portal_home;
use crate::portal_page::PortalPage;
use crate::portal_typography;
use crate::windows::dir_window::ui_dir_windows;
use crate::windows::resources_window::resources_ui;
use crate::windows::selected_items_window::selected_items_ui;
use crate::windows::starting_points_window::starting_points_ui;
use crate::windows::work_dirs_window::DNDResponse;
use crate::windows::work_dirs_window::work_dirs_ui;
use eframe::egui::Align2;
use eframe::egui::CentralPanel;
use eframe::egui::Color32;
use eframe::egui::Frame;
use eframe::egui::Margin;
use eframe::egui::RichText;
use eframe::egui::ScrollArea;
use eframe::egui::Ui;
use eframe::egui::Window;

impl MyApp {
    pub fn draw_app(&mut self, ui: &mut Ui) {
        self.portal_header(ui);
        self.portal_navigation(ui);
        CentralPanel::default()
            .frame(Frame::new().fill(Color32::WHITE).inner_margin(Margin::symmetric(32, 24)))
            .show(ui, |ui| {
                if self.page != PortalPage::Home {
                    ui.label(RichText::new(self.page.title()).strong().size(22.0));
                    ui.add_space(20.0);
                }
                match self.page {
                    PortalPage::Home => portal_home(self, ui),
                    PortalPage::Logs => {
                        self.log_view.show(ui, &self.log_capture);
                        // UI refresh is CT policy; capture has no dependency on a window.
                        ui.ctx().request_repaint_after(std::time::Duration::from_millis(250));
                    }
                    PortalPage::Resources
                    | PortalPage::Subscriptions
                    | PortalPage::ResourceGroups
                    | PortalPage::StorageAccounts
                    | PortalPage::ContainerRegistries
                    | PortalPage::KeyVaults => resources_ui(self, ui),
                    PortalPage::StartingPoints => starting_points_ui(self, ui),
                    PortalPage::SelectedItems => selected_items_ui(self, ui),
                    PortalPage::WorkDirs => {
                        let mut response = None;
                        work_dirs_ui(self, ui, &mut response);
                        if let Some(DNDResponse { source_index, destination_index }) = response {
                            if let Some(item) = self.work_dirs_config.work_dirs.remove_index(source_index) {
                                let destination_index = destination_index
                                    .saturating_sub(usize::from(source_index < destination_index))
                                    .min(self.work_dirs_config.work_dirs.len());
                                self.work_dirs_config.work_dirs.shift_insert(destination_index, item);
                            } else {
                                tracing::warn!(source_index, "Dragged work directory no longer exists");
                            }
                        }
                    }
                    PortalPage::FileDrop => {
                        ui.label("Drop Terraform files or directories into this window to add work directories.");
                        file_drag_ui(self, ui);
                    }
                }
            });
        ui_dir_windows(self, ui.ctx());

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
