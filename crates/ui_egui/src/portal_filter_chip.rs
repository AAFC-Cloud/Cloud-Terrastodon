use eframe::egui::Button;
use eframe::egui::Color32;
use eframe::egui::CornerRadius;
use eframe::egui::PopupCloseBehavior;
use eframe::egui::Stroke;
use eframe::egui::Ui;
use eframe::egui::Vec2;
use eframe::egui::WidgetText;
use eframe::egui::containers::menu::MenuButton;
use eframe::egui::containers::menu::MenuConfig;
use std::sync::Arc;

/// A filter menu that stays open while users choose multiple values.
pub(crate) fn portal_filter_chip(
    ui: &mut Ui,
    label: impl Into<WidgetText>,
    add_contents: impl FnOnce(&mut Ui),
) {
    let menu_style = Arc::clone(ui.style());
    ui.scope(|ui| {
        ui.spacing_mut().button_padding = Vec2::new(12.0, 5.0);
        let visuals = ui.visuals_mut();
        visuals.override_text_color = Some(Color32::from_rgb(41, 40, 39));
        for (style, fill) in [
            (
                &mut visuals.widgets.inactive,
                Color32::from_rgb(225, 239, 248),
            ),
            (
                &mut visuals.widgets.hovered,
                Color32::from_rgb(205, 228, 245),
            ),
            (
                &mut visuals.widgets.active,
                Color32::from_rgb(190, 219, 239),
            ),
        ] {
            style.bg_fill = fill;
            style.weak_bg_fill = fill;
            style.expansion = 0.0;
            style.bg_stroke = Stroke::NONE;
        }
        MenuButton::from_button(Button::new(label.into()).corner_radius(CornerRadius::same(14)))
            .config(MenuConfig::new().close_behavior(PopupCloseBehavior::CloseOnClickOutside))
            .ui(ui, |ui| {
                ui.set_style(menu_style);
                add_contents(ui);
            });
    });
}
