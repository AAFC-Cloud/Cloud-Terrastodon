use crate::app::MyApp;
use crate::portal_browser_control::PortalBrowserControl;
use crate::portal_page::PortalPage;
use crate::portal_tab_target::PortalTabTarget;
use eframe::egui::Ui;

impl MyApp {
    /// Render browser controls and the address for the current tab.
    pub(crate) fn show_portal_browser_toolbar(&mut self, ui: &mut Ui, target: &PortalTabTarget) {
        let (visits, current) = self.tabs.history();
        let can_go_back = current > 0;
        let can_go_forward = current + 1 < visits.len();
        let can_refresh = self.can_refresh_portal_view(target);
        let can_go_home = *target != PortalTabTarget::Page(PortalPage::Home);

        let action = ui
            .horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                let mut action = None;
                if PortalBrowserControl::Back.show(ui, can_go_back) {
                    action = Some(PortalBrowserControl::Back);
                }
                if can_go_forward && PortalBrowserControl::Forward.show(ui, true) {
                    action = Some(PortalBrowserControl::Forward);
                }
                if PortalBrowserControl::Refresh.show(ui, can_refresh) {
                    action = Some(PortalBrowserControl::Refresh);
                }
                if PortalBrowserControl::Home.show(ui, can_go_home) {
                    action = Some(PortalBrowserControl::Home);
                }
                self.show_portal_address(ui, target);
                action
            })
            .inner;

        let moved = match action {
            Some(PortalBrowserControl::Back) => self.tabs.back(),
            Some(PortalBrowserControl::Forward) => self.tabs.forward(),
            Some(PortalBrowserControl::Refresh) => {
                self.refresh_portal_view(ui, target);
                return;
            }
            Some(PortalBrowserControl::Home) => {
                self.navigate_current_tab(PortalTabTarget::Page(PortalPage::Home));
                return;
            }
            None => return,
        };
        if moved {
            self.after_history_navigation();
        }
    }
}
