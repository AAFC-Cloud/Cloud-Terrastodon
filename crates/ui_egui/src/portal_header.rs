use crate::app::MyApp;
use crate::portal_page::PortalPage;
use crate::portal_tab_target::PortalTabTarget;
use eframe::egui::Align;
use eframe::egui::Color32;
use eframe::egui::CornerRadius;
use eframe::egui::Frame;
use eframe::egui::Id;
use eframe::egui::Key;
use eframe::egui::Layout;
use eframe::egui::Modifiers;
use eframe::egui::Panel;
use eframe::egui::RichText;
use eframe::egui::Sense;
use eframe::egui::TextEdit;
use eframe::egui::TextStyle;
use eframe::egui::Ui;
use eframe::egui::Vec2;
use eframe::egui::WidgetInfo;
use eframe::egui::WidgetType;

impl MyApp {
    /// The shared portal header keeps search and the active identity visible on every page.
    pub fn portal_header(&mut self, ui: &mut Ui) {
        let focus_search = ui.input_mut(|input| input.consume_key(Modifiers::CTRL, Key::K));
        Panel::top("portal_header")
            .exact_size(40.0)
            .frame(Frame::new().fill(Color32::from_rgb(0x00, 0x78, 0xd4)))
            .show_separator_line(false)
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.spacing_mut().button_padding = Vec2::new(12.0, 0.0);
                ui.spacing_mut().interact_size.y = 40.0;
                let visuals = ui.visuals_mut();
                visuals.override_text_color = Some(Color32::WHITE);
                visuals.widgets.inactive.bg_fill = Color32::TRANSPARENT;
                visuals.widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
                visuals.widgets.inactive.bg_stroke = Default::default();
                for widget in [
                    &mut visuals.widgets.hovered,
                    &mut visuals.widgets.active,
                    &mut visuals.widgets.open,
                ] {
                    widget.bg_fill = Color32::from_rgb(0x10, 0x6e, 0xbe);
                    widget.weak_bg_fill = widget.bg_fill;
                    widget.bg_stroke = Default::default();
                    widget.corner_radius = CornerRadius::ZERO;
                    widget.expansion = 0.0;
                }

                ui.horizontal(|ui| {
                    let (rect, response) =
                        ui.allocate_exact_size(Vec2::splat(40.0), Sense::click());
                    response.widget_info(|| {
                        WidgetInfo::labeled(
                            WidgetType::Button,
                            ui.is_enabled(),
                            "Cloud Terrastodon home",
                        )
                    });
                    if response.hovered() || response.has_focus() {
                        ui.painter()
                            .rect_filled(rect, 0.0, Color32::from_rgb(0x10, 0x6e, 0xbe));
                    }
                    for x in [-6.0, 0.0, 6.0] {
                        for y in [-6.0, 0.0, 6.0] {
                            ui.painter().circle_filled(
                                rect.center() + Vec2::new(x, y),
                                1.25,
                                Color32::WHITE,
                            );
                        }
                    }
                    if response.on_hover_text("Cloud Terrastodon home").clicked() {
                        self.navigate(PortalPage::Home);
                    }
                    // Keep enough room for the identity and search on smaller windows.
                    if ui.available_width() > 660.0 {
                        ui.allocate_ui_with_layout(
                            Vec2::new(208.0, 40.0),
                            Layout::left_to_right(Align::Center),
                            |ui| {
                                ui.label(
                                    RichText::new("Cloud Terrastodon")
                                        .color(Color32::WHITE)
                                        .size(17.0)
                                        .variation("wght", 600.0)
                                        .variation("opsz", 17.0),
                                );
                            },
                        );
                    }
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if let Some(action) = self.account_picker.show(
                            ui,
                            self.authentication,
                            &self.account_profiles,
                            self.account_operation.is_some(),
                            &mut self.egui_config.demo_mode,
                        ) {
                            self.handle_account_picker_action(action, ui.ctx());
                        }
                        ui.add_space(16.0);
                        let search_space = (ui.available_width() - 16.0).max(0.0);
                        let search_width = search_space.min(560.0);
                        ui.add_space((search_space - search_width) * 0.5);
                        ui.scope(|ui| {
                            ui.spacing_mut().interact_size.y = 28.0;
                            ui.visuals_mut().override_text_color =
                                Some(Color32::from_rgb(50, 49, 48));
                            ui.visuals_mut().text_edit_bg_color = Some(Color32::WHITE);
                            ui.visuals_mut().widgets.inactive.bg_fill = Color32::WHITE;
                            let response = ui.add_sized(
                                [search_width, 28.0],
                                TextEdit::singleline(&mut self.resource_search)
                                    .id(Id::new("portal_resource_search"))
                                    .font(TextStyle::Body)
                                    .margin(Vec2::new(10.0, 5.0))
                                    .hint_text("Search resources (Ctrl+K)"),
                            );
                            if focus_search {
                                response.request_focus();
                            }
                            if response.changed() {
                                self.tabs.open(PortalTabTarget::Page(PortalPage::Resources));
                                self.retain_open_tab_data();
                            }
                        });
                    });
                });
            });
    }
}
