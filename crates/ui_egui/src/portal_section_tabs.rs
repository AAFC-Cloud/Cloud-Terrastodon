use eframe::egui::Button;
use eframe::egui::Color32;
use eframe::egui::CursorIcon;
use eframe::egui::RichText;
use eframe::egui::Stroke;
use eframe::egui::Ui;
use eframe::egui::Vec2;
use eframe::egui::WidgetInfo;
use eframe::egui::WidgetType;

pub(crate) const SECTION_TAB_HEIGHT: f32 = 30.0;

/// Flat page tabs with an underline for the selected section.
pub(crate) fn portal_section_tabs<T: Copy + PartialEq>(
    ui: &mut Ui,
    selected: &mut T,
    choices: impl IntoIterator<Item = T>,
    label: impl Fn(T) -> &'static str,
) {
    ui.scope(|ui| {
        let hovered = &mut ui.visuals_mut().widgets.hovered;
        hovered.weak_bg_fill = Color32::from_rgb(240, 244, 248);
        hovered.bg_stroke = Stroke::NONE;
        for choice in choices {
            let active = choice == *selected;
            let label = label(choice);
            let text = RichText::new(label).size(13.0);
            let response = ui
                .add(
                    Button::new(if active { text.strong() } else { text })
                        .min_size(Vec2::new(0.0, SECTION_TAB_HEIGHT))
                        .frame_when_inactive(false),
                )
                .on_hover_cursor(CursorIcon::PointingHand);
            response.widget_info(|| {
                WidgetInfo::selected(WidgetType::SelectableLabel, ui.is_enabled(), active, label)
            });
            if active {
                ui.painter().hline(
                    response.rect.x_range(),
                    response.rect.bottom() - 1.0,
                    Stroke::new(2.0, Color32::from_rgb(0, 120, 212)),
                );
            }
            if response.clicked() {
                *selected = choice;
            }
        }
    });
}
