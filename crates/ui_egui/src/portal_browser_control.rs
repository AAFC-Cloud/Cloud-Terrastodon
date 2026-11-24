use eframe::egui::CursorIcon;
use eframe::egui::Painter;
use eframe::egui::Pos2;
use eframe::egui::Response;
use eframe::egui::Sense;
use eframe::egui::Shape;
use eframe::egui::Stroke;
use eframe::egui::Ui;
use eframe::egui::Vec2;
use eframe::egui::WidgetInfo;
use eframe::egui::WidgetType;

#[derive(Clone, Copy)]
pub(crate) enum PortalBrowserControl {
    Back,
    Forward,
    Refresh,
    Home,
    History,
}

impl PortalBrowserControl {
    pub(crate) const SIZE: f32 = 32.0;

    pub(crate) fn show(self, ui: &mut Ui, enabled: bool) -> Response {
        let label = match self {
            Self::Back => "Back",
            Self::Forward => "Forward",
            Self::Refresh => "Refresh this view",
            Self::Home => "Home",
            Self::History => "History (Ctrl+H)",
        };

        ui.push_id(label, |ui| {
            ui.add_enabled_ui(enabled, |ui| {
                let (rect, response) =
                    ui.allocate_exact_size(Vec2::splat(Self::SIZE), Sense::click());
                response.widget_info(|| {
                    WidgetInfo::labeled(WidgetType::Button, response.enabled(), label)
                });

                let center = rect.center();
                let painter = ui.painter();
                let visuals = ui.style().interact(&response);
                if response.hovered() || response.is_pointer_button_down_on() {
                    painter.circle_filled(center, 15.0, visuals.weak_bg_fill);
                }
                if response.has_focus() {
                    painter.circle_stroke(center, 15.0, visuals.fg_stroke);
                }
                self.paint(painter, center, Stroke::new(1.5, visuals.fg_stroke.color));

                if response.enabled() {
                    response
                        .on_hover_cursor(CursorIcon::PointingHand)
                        .on_hover_text(label)
                } else {
                    response.on_disabled_hover_text(label)
                }
            })
            .inner
        })
        .inner
    }

    fn paint(self, painter: &Painter, center: Pos2, stroke: Stroke) {
        let point = |x, y| center + Vec2::new(x, y);
        match self {
            Self::Back | Self::Forward => {
                let direction = if matches!(self, Self::Back) {
                    -1.0
                } else {
                    1.0
                };
                let tip = point(6.5 * direction, 0.0);
                painter.line_segment([point(-6.5 * direction, 0.0), tip], stroke);
                painter.add(Shape::line(
                    vec![
                        point(1.5 * direction, -5.0),
                        tip,
                        point(1.5 * direction, 5.0),
                    ],
                    stroke,
                ));
            }
            Self::Refresh => {
                let arc = (0..=24)
                    .map(|step| {
                        let angle = (25.0_f32 + 305.0 * step as f32 / 24.0).to_radians();
                        center + Vec2::new(angle.cos(), angle.sin()) * 6.3
                    })
                    .collect();
                painter.add(Shape::line(arc, stroke));
                painter.add(Shape::line(
                    vec![point(5.5, -7.5), point(5.5, -3.2), point(1.2, -3.2)],
                    stroke,
                ));
            }
            Self::Home => {
                painter.add(Shape::line(
                    vec![point(-7.0, -0.5), point(0.0, -6.5), point(7.0, -0.5)],
                    stroke,
                ));
                painter.add(Shape::line(
                    vec![
                        point(-5.5, -1.8),
                        point(-5.5, 6.0),
                        point(-1.8, 6.0),
                        point(-1.8, 1.5),
                        point(1.8, 1.5),
                        point(1.8, 6.0),
                        point(5.5, 6.0),
                        point(5.5, -1.8),
                    ],
                    stroke,
                ));
            }
            Self::History => {
                let arc = (0..=24)
                    .map(|step| {
                        let angle = (143.0_f32 - 300.0 * step as f32 / 24.0).to_radians();
                        center + Vec2::new(angle.cos(), angle.sin()) * 6.5
                    })
                    .collect();
                painter.add(Shape::line(arc, stroke));
                painter.add(Shape::line(
                    vec![point(-6.0, -6.5), point(-6.0, -2.5), point(-2.0, -2.5)],
                    stroke,
                ));
                painter.add(Shape::line(
                    vec![point(0.0, -3.5), center, point(2.5, 1.8)],
                    stroke,
                ));
            }
        }
    }
}
