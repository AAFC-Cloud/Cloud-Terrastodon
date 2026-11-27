use eframe::egui::Align;
use eframe::egui::AsId;
use eframe::egui::Frame;
use eframe::egui::Id;
use eframe::egui::Label;
use eframe::egui::Layout;
use eframe::egui::Panel;
use eframe::egui::RichText;
use eframe::egui::Spinner;
use eframe::egui::Stroke;
use eframe::egui::Ui;
use egui_extras::TableBuilder;

pub(crate) fn show_inventory_table(ui: &mut Ui, contents: impl FnOnce(TableBuilder<'_>)) {
    // egui_extras uses this stroke for idle column handles. Use the same Ui
    // so the table retains its stored column widths and scroll position.
    let original_stroke = ui.visuals().widgets.noninteractive.bg_stroke;
    ui.visuals_mut().widgets.noninteractive.bg_stroke = Stroke::NONE;
    contents(
        TableBuilder::new(ui)
            .striped(false)
            .resizable(true)
            .min_scrolled_height(0.0)
            .cell_layout(Layout::left_to_right(Align::Center)),
    );
    ui.visuals_mut().widgets.noninteractive.bg_stroke = original_stroke;
}

/// Reserve a footer before the table consumes the remaining viewport height.
pub(crate) fn show_inventory_result_count(ui: &mut Ui, id: impl AsId, shown: usize, total: usize) {
    show_inventory_footer(ui, id, shown, total, None);
}

pub(crate) fn show_inventory_footer(
    ui: &mut Ui,
    id: impl AsId,
    shown: usize,
    total: usize,
    status: Option<&str>,
) {
    let id = ui.make_persistent_id(("inventory-result-count", Id::new(id)));
    Panel::bottom(id)
        .exact_size(28.0)
        .frame(Frame::NONE)
        .show(ui, |ui| {
            ui.add_space(7.0);
            ui.horizontal(|ui| {
                ui.add(
                    Label::new(
                        RichText::new(format!("Showing {shown} of {total} entries")).small(),
                    )
                    .truncate(),
                );
                if let Some(status) = status {
                    ui.add(Spinner::new().size(12.0));
                    ui.add(Label::new(RichText::new(status).weak().small()).truncate())
                        .on_hover_text(status);
                }
            });
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::Context;
    use eframe::egui::Pos2;
    use eframe::egui::RawInput;
    use eframe::egui::Rect;
    use eframe::egui::Shape;
    use eframe::egui::Vec2;
    use egui_extras::Column;

    #[test]
    fn long_inventory_leaves_room_for_the_count_in_a_short_window() {
        let input = RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(480.0, 180.0))),
            ..Default::default()
        };
        let mut table_bottom = 0.0;
        let mut output = Context::default().run_ui(input, |ui| {
            ui.heading("Inventory");
            show_inventory_result_count(ui, "test-inventory", 1250, 2000);
            show_inventory_table(ui, |table| {
                let body = table
                    .column(Column::remainder())
                    .header(28.0, |mut header| {
                        header.col(|ui| {
                            ui.strong("Name");
                        });
                    })
                    .body(|body| {
                        body.rows(28.0, 1250, |mut row| {
                            row.col(|ui| {
                                ui.label("Resource");
                            });
                        });
                    });
                table_bottom = body.inner_rect.bottom();
            });
        });
        output.textures_delta.clear();
        let footer = output
            .shapes
            .iter()
            .find_map(|clipped| match &clipped.shape {
                Shape::Text(text) if text.galley.text() == "Showing 1250 of 2000 entries" => {
                    Some(text)
                }
                _ => None,
            });
        assert!(footer.is_some_and(|footer| footer.pos.y > table_bottom));
    }
}
