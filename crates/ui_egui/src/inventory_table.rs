use eframe::egui::Align;
use eframe::egui::Layout;
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
            .cell_layout(Layout::left_to_right(Align::Center)),
    );
    ui.visuals_mut().widgets.noninteractive.bg_stroke = original_stroke;
}
