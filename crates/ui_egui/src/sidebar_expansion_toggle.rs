use eframe::egui::Button;
use eframe::egui::Frame;
use eframe::egui::Panel;
use eframe::egui::Ui;
use eframe::egui::WidgetInfo;
use eframe::egui::WidgetType;

/// Reserve a footer for the toggle before laying out the sidebar's contents.
pub(crate) fn sidebar_expansion_toggle(ui: &mut Ui, expanded: &mut bool, label: &str) {
    Panel::bottom(ui.make_persistent_id("sidebar_expansion_toggle"))
        .exact_size(32.0)
        .resizable(false)
        .show_separator_line(false)
        .frame(Frame::NONE)
        .show(ui, |ui| {
            let (arrow, action) = if *expanded {
                ("«", "Collapse")
            } else {
                ("»", "Expand")
            };
            let label = format!("{action} {label}");
            let response = ui.add_sized(ui.available_size(), Button::new(arrow).frame(false));
            response
                .widget_info(|| WidgetInfo::labeled(WidgetType::Button, ui.is_enabled(), &label));
            if response.on_hover_text(label).clicked() {
                *expanded = !*expanded;
                ui.ctx().request_repaint();
            }
        });
}
