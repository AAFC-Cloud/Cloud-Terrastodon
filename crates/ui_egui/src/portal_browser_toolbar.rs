use crate::app::MyApp;
use crate::portal_browser_control::PortalBrowserControl;
use crate::portal_page::PortalPage;
use crate::portal_tab_open::PortalTabOpen;
use crate::portal_tab_target::PortalTabTarget;
use eframe::egui::Align;
use eframe::egui::Key;
use eframe::egui::Layout;
use eframe::egui::Modifiers;
use eframe::egui::PointerButton;
use eframe::egui::Ui;
use eframe::egui::Vec2;

impl MyApp {
    pub(crate) fn handle_history_input(&mut self, ui: &Ui) {
        if ui.input_mut(|input| input.consume_key(Modifiers::CTRL, Key::H)) {
            self.navigate(PortalPage::History);
            return;
        }
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
        self.azure.resource_search.clear();
        self.retain_open_tab_data();
    }

    /// Render browser navigation and the address and resource search field.
    pub(crate) fn show_portal_browser_toolbar(&mut self, ui: &mut Ui, target: &PortalTabTarget) {
        let focus_search = ui.input_mut(|input| {
            // Ctrl+Shift+K duplicates the active tab.
            !input.modifiers.shift && input.consume_key(Modifiers::CTRL, Key::K)
        });
        let (visits, current) = self.tabs.history();
        let can_go_back = current > 0;
        let can_go_forward = current + 1 < visits.len();
        let can_refresh = self.can_refresh_portal_view(target);
        let can_go_home = *target != PortalTabTarget::Page(PortalPage::Home);
        let can_open_history = *target != PortalTabTarget::Page(PortalPage::History);

        let action = ui
            .allocate_ui_with_layout(
                Vec2::new(ui.available_width(), PortalBrowserControl::SIZE),
                Layout::left_to_right(Align::Center),
                |ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    let mut action = None;
                    for (control, enabled) in [
                        (PortalBrowserControl::Back, can_go_back),
                        (PortalBrowserControl::Forward, can_go_forward),
                        (PortalBrowserControl::Refresh, can_refresh),
                        (PortalBrowserControl::Home, can_go_home),
                    ] {
                        if matches!(control, PortalBrowserControl::Forward) && !enabled {
                            continue;
                        }
                        let response = control.show(ui, enabled);
                        if response.clicked() || response.middle_clicked() {
                            action = Some((control, response));
                        }
                    }
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let response = PortalBrowserControl::History.show(ui, can_open_history);
                        if response.clicked() || response.middle_clicked() {
                            action = Some((PortalBrowserControl::History, response));
                        }
                        ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                            if self.show_portal_address(ui, target, focus_search) {
                                let search = self.azure.resource_search.clone();
                                self.navigate(PortalPage::Resources);
                                self.azure.resource_search = search;
                            }
                        });
                    });
                    action
                },
            )
            .inner;

        let Some((control, response)) = action else {
            return;
        };
        if !matches!(control, PortalBrowserControl::Refresh)
            && PortalTabOpen::opens_in_background(&response)
        {
            let (visits, current) = self.tabs.history();
            let destination = match control {
                PortalBrowserControl::Back => current
                    .checked_sub(1)
                    .and_then(|position| visits.get(position))
                    .cloned(),
                PortalBrowserControl::Forward => visits.get(current + 1).cloned(),
                PortalBrowserControl::Home => Some(PortalTabTarget::Page(PortalPage::Home)),
                PortalBrowserControl::History => Some(PortalTabTarget::Page(PortalPage::History)),
                PortalBrowserControl::Refresh => None,
            };
            if let Some(destination) = destination {
                self.open_background_tab(destination);
            }
            return;
        }
        let moved = match control {
            PortalBrowserControl::Back => self.tabs.back(),
            PortalBrowserControl::Forward => self.tabs.forward(),
            PortalBrowserControl::Refresh => {
                if response.clicked() {
                    self.refresh_portal_view(ui, target);
                }
                return;
            }
            PortalBrowserControl::Home => {
                self.navigate_current_tab(PortalTabTarget::Page(PortalPage::Home));
                return;
            }
            PortalBrowserControl::History => {
                self.navigate(PortalPage::History);
                return;
            }
        };
        if moved {
            self.after_history_navigation();
        }
    }
}
