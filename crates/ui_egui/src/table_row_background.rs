use eframe::egui::Color32;
use eframe::egui::CornerRadius;
use eframe::egui::Painter;
use eframe::egui::Shape;
use eframe::egui::Stroke;
use eframe::egui::Ui;
use eframe::egui::Vec2;
use eframe::egui::emath::GuiRounding as _;
use egui_extras::TableBody;
use egui_extras::TableRow;

/// Draws a continuous hover and selection background behind a table's cells.
pub(crate) struct TableRowBackground {
    painter: Painter,
    width: f32,
    half_spacing: Vec2,
}

impl TableRowBackground {
    pub(crate) fn new(body: &mut TableBody<'_>) -> Self {
        let widths = body.widths();
        let column_width = widths.iter().sum::<f32>();
        let column_gaps = widths.len().saturating_sub(1);
        let ui = body.ui_mut();
        let spacing = ui.spacing().item_spacing;
        Self {
            painter: ui.painter().clone(),
            width: column_width + spacing.x * column_gaps as f32,
            half_spacing: spacing * 0.5,
        }
    }

    /// The first cell supplies the row's position and its selection after handling input.
    pub(crate) fn first_cell(
        &self,
        row: &mut TableRow<'_, '_>,
        add_contents: impl FnOnce(&mut Ui) -> bool,
    ) {
        row.col(|ui| {
            let mut rect = ui.max_rect();
            rect.set_width(self.width);
            let rect = rect.expand2(self.half_spacing).round_ui();
            let mut hover_rect = rect.intersect(self.painter.clip_rect());
            // Give the shared boundary to the next row when the pointer sits on it.
            hover_rect.max.y = hover_rect.max.y.next_down();
            let hovered = self
                .painter
                .ctx()
                .rect_contains_pointer(self.painter.layer_id(), hover_rect);

            // Reserve the paint order before the checkbox or link draws over the background.
            let background = self.painter.add(Shape::Noop);
            let selected = add_contents(ui);
            let gray = match (selected, hovered) {
                (false, false) => None,
                (false, true) => Some(244),
                (true, false) => Some(229),
                (true, true) => Some(213),
            };
            if let Some(gray) = gray {
                self.painter.set(
                    background,
                    Shape::rect_filled(rect, CornerRadius::ZERO, Color32::from_gray(gray)),
                );
            }
            self.painter.hline(
                rect.x_range(),
                rect.bottom(),
                Stroke::new(1.0, Color32::from_gray(202)),
            );
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory_table::show_inventory_table;
    use eframe::egui::Checkbox;
    use eframe::egui::Context;
    use eframe::egui::Event;
    use eframe::egui::Pos2;
    use eframe::egui::RawInput;
    use egui_extras::Column;

    #[test]
    fn hover_in_a_later_cell_shades_the_entire_selectable_row() {
        let context = Context::default();
        let draw = |mut selected, pointer: Option<Pos2>| {
            let mut checkbox = Pos2::ZERO;
            let mut name = Pos2::ZERO;
            let input = RawInput {
                events: pointer.map(Event::PointerMoved).into_iter().collect(),
                ..Default::default()
            };
            let mut output = context.run_ui(input, |ui| {
                let surrounding_stroke = Stroke::new(1.0, Color32::RED);
                ui.visuals_mut().widgets.noninteractive.bg_stroke = surrounding_stroke;
                show_inventory_table(ui, |table| {
                    table
                        .column(Column::initial(32.0).clip(true))
                        .column(Column::exact(140.0))
                        .body(|mut body| {
                            let background = TableRowBackground::new(&mut body);
                            body.rows(28.0, 1, |mut row| {
                                background.first_cell(&mut row, |ui| {
                                    checkbox =
                                        ui.add(Checkbox::without_text(&mut selected)).rect.center();
                                    selected
                                });
                                row.col(|ui| name = ui.link("Example").rect.center());
                            });
                        });
                });
                assert_eq!(
                    ui.visuals().widgets.noninteractive.bg_stroke,
                    surrounding_stroke
                );
            });
            // This test inspects shapes without uploading textures to a renderer.
            output.textures_delta.clear();
            assert!(name.x > checkbox.x);
            assert!((checkbox.y - name.y).abs() < 0.5);
            assert!(output.shapes.iter().any(|shape| {
                matches!(&shape.shape, Shape::LineSegment { points: [left, right], .. }
                    if left.y == right.y && left.y > checkbox.y && left.x <= checkbox.x && right.x >= name.x)
            }));
            assert!(!output.shapes.iter().any(|shape| {
                matches!(&shape.shape, Shape::LineSegment { points: [top, bottom], stroke }
                    if !stroke.is_empty() && top.x == bottom.x && top.x > checkbox.x && top.x < name.x
                        && top.y < checkbox.y && bottom.y > checkbox.y)
            }));
            let gray = output.shapes.iter().find_map(|shape| match &shape.shape {
                Shape::Rect(fill)
                    if fill.fill != Color32::TRANSPARENT
                        && fill.rect.contains(checkbox)
                        && fill.rect.contains(name)
                        && shape.clip_rect.contains(checkbox)
                        && shape.clip_rect.contains(name)
                        && fill.rect.height() < 40.0 =>
                {
                    Some(fill.fill.r())
                }
                _ => None,
            });
            (gray, name)
        };

        let (normal, name) = draw(false, None);
        let (selected, _) = draw(true, None);
        let (hovered, _) = draw(false, Some(name));
        let (selected_hovered, _) = draw(true, Some(name));
        assert!(normal.is_none());
        assert!(hovered.unwrap() > selected.unwrap());
        assert!(selected.unwrap() > selected_hovered.unwrap());
    }
}
