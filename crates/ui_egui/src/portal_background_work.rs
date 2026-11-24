use crate::app::MyApp;
use crate::inventory_table::show_inventory_table;
use eframe::egui::Label;
use eframe::egui::Ui;
use egui_extras::Column;
use teamy_tracing::gui::escape_display;

impl MyApp {
    pub(crate) fn show_background_work(&self, ui: &mut Ui) {
        let pending = self.pending_principal_lookups();
        if pending != 0 {
            ui.label(format!("IAM principal lookups pending · {pending}"));
        }

        let work = match self.work_tracker.active_work() {
            Ok(work) if work.is_empty() => {
                ui.weak("No work in flight.");
                return;
            }
            Ok(work) => work,
            Err(error) => {
                ui.weak(format!("Background activity unavailable · {error}"));
                return;
            }
        };
        const HEADER_HEIGHT: f32 = 24.0;
        let height = (ui.available_height() - HEADER_HEIGHT - ui.spacing().item_spacing.y).max(0.0);
        show_inventory_table(ui, |table| {
            table
                .id_salt("active-work")
                .max_scroll_height(height)
                .column(Column::remainder().clip(true))
                .column(Column::initial(90.0).at_least(75.0).clip(true))
                .header(HEADER_HEIGHT, |mut header| {
                    header.col(|ui| {
                        ui.strong("Operation");
                    });
                    header.col(|ui| {
                        ui.strong("Elapsed");
                    });
                })
                .body(|body| {
                    body.rows(26.0, work.len(), |mut row| {
                        let job = &work[row.index()];
                        row.col(|ui| {
                            let description = escape_display(&job.description);
                            ui.add(Label::new(&description).truncate())
                                .on_hover_text(description);
                        });
                        row.col(|ui| {
                            ui.label(format!("{:.1} s", job.started_at.elapsed().as_secs_f64()));
                        });
                    });
                });
        });
    }
}
