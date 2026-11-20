use crate::app::MyApp;
use crate::portal_browser_control::PortalBrowserControl;
use crate::portal_page::PortalPage;
use crate::portal_tab_target::PortalTabTarget;
use eframe::egui::Key;
use eframe::egui::Modifiers;
use eframe::egui::PointerButton;
use eframe::egui::Ui;

impl MyApp {
    pub(crate) fn handle_history_input(&mut self, ui: &Ui) {
        let (back, forward) = ui.input_mut(|input| {
            let back = input.consume_key(Modifiers::ALT, Key::ArrowLeft)
                || input.pointer.button_pressed(PointerButton::Extra1);
            let forward = input.consume_key(Modifiers::ALT, Key::ArrowRight)
                || input.pointer.button_pressed(PointerButton::Extra2);
            (back, forward)
        });
        if (back && self.tabs.back()) || (forward && self.tabs.forward()) {
            self.after_history_navigation();
        }
    }

    pub(crate) fn after_history_navigation(&mut self) {
        self.resource_search.clear();
        self.retain_open_tab_data();
    }

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
