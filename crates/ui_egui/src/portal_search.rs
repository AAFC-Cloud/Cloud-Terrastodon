use eframe::egui::Align;
use eframe::egui::Atom;
use eframe::egui::Color32;
use eframe::egui::CornerRadius;
use eframe::egui::FontId;
use eframe::egui::Id;
use eframe::egui::Margin;
use eframe::egui::Response;
use eframe::egui::Stroke;
use eframe::egui::TextEdit;
use eframe::egui::Ui;
use eframe::egui::Vec2;

/// Render a desktop search or filter input with a consistent focus indicator.
pub(crate) fn portal_search(ui: &mut Ui, value: &mut String, hint: &str, width: f32) -> Response {
    let id = ui.next_auto_id();
    portal_search_with_id(ui, id, value, hint, width)
}

/// Keep input focus and cursor state when surrounding widgets change.
pub(crate) fn portal_search_with_id(
    ui: &mut Ui,
    id: Id,
    value: &mut String,
    hint: &str,
    width: f32,
) -> Response {
    ui.scope(|ui| {
        let ink = Color32::from_rgb(41, 40, 39);
        let visuals = ui.visuals_mut();
        visuals.text_edit_bg_color = Some(Color32::WHITE);
        visuals.override_text_color = Some(ink);
        visuals.selection.stroke = Stroke::new(1.0, Color32::from_rgb(0, 120, 212));
        for (widget, border) in [
            (
                &mut visuals.widgets.inactive,
                Color32::from_rgb(138, 136, 134),
            ),
            (&mut visuals.widgets.hovered, ink),
            (&mut visuals.widgets.active, ink),
        ] {
            widget.corner_radius = CornerRadius::same(2);
            widget.expansion = 0.0;
            widget.bg_stroke = Stroke::new(1.0, border);
        }
        let height = ui.spacing().interact_size.y.max(24.0);
        let width = width.min(ui.available_width()).max(1.0);
        let icon = Id::new("search-icon");
        let edit = TextEdit::singleline(value)
            .id(id)
            .prefix(Atom::custom(icon, Vec2::splat(13.0)))
            .hint_text(hint)
            .text_color(ink)
            .font(FontId::proportional(13.0))
            .vertical_align(Align::Center)
            .margin(Margin::symmetric(7, 3))
            .desired_width(width)
            .min_size(Vec2::new(0.0, height))
            .show(ui);
        if let Some(rect) = edit.response.rect(icon) {
            let painter = ui.painter_at(edit.response.rect);
            let center = rect.center() - Vec2::splat(1.0);
            let stroke = Stroke::new(1.2, Color32::from_gray(130));
            painter.circle_stroke(center, 3.5, stroke);
            painter.line_segment(
                [center + Vec2::splat(2.5), center + Vec2::splat(5.5)],
                stroke,
            );
        }
        edit.response.response
    })
    .inner
}
