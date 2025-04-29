use crate::app::MyApp;
use eframe::egui::Color32;
use eframe::egui::Context;
use eframe::egui::DragAndDrop;
use eframe::egui::Frame;
use eframe::egui::Id;
use eframe::egui::Pos2;
use eframe::egui::Rect;
use eframe::egui::Stroke;
use eframe::egui::Ui;
use eframe::egui::Vec2;
use eframe::egui::Window;
use std::path::PathBuf;
use tracing::info;

pub fn draw_work_dirs_window(app: &mut MyApp, ctx: &Context) {
    let mut dnd_response = None;

    // draw window
    Window::new("TF Work Dirs").show(ctx, |ui| {
        let frame = Frame::default().inner_margin(4.0);
        let old = ui.visuals_mut().widgets.inactive.bg_fill;
        ui.visuals_mut().widgets.inactive.bg_fill = Color32::TRANSPARENT;
        let (_, dropped_payload) = ui.dnd_drop_zone::<usize, ()>(frame, |ui| {
            let work_dirs = app.work_dirs_config.work_dirs.clone();
            if work_dirs.is_empty() {
                ui.label("There's nothing here!");
            } else {
                ui.vertical(|ui| {
                    for (i, work_dir) in work_dirs.into_iter().enumerate() {
                        if let Some(response) = ui_work_dir_row(ui, app, i, &work_dir) {
                            dnd_response = Some(response);
                        }
                    }
                });
            }
        });
        ui.visuals_mut().widgets.inactive.bg_fill = old;
        if let Some(source_index) = dropped_payload {
            // Dropped onto the area but not on any one item
            dnd_response = Some(DNDResponse {
                source_index: *source_index,
                destination_index: app.work_dirs_config.work_dirs.len(),
            });
        }
    });

    // handle drag and drop logic
    if let Some(DNDResponse {
        source_index,
        mut destination_index,
    }) = dnd_response
    {
        destination_index -= (source_index < destination_index) as usize;
        let item = app
            .work_dirs_config
            .work_dirs
            .remove_index(source_index)
            .unwrap();
        app.work_dirs_config
            .work_dirs
            .shift_insert(destination_index, item);
    } else if DragAndDrop::has_payload_of_type::<usize>(ctx)
        && ctx.input(|input_state| input_state.pointer.any_released())
    {
        let source_index = DragAndDrop::take_payload::<usize>(ctx).unwrap();
        let work_dir = app.work_dirs_config.work_dirs[*source_index].clone();
        let pointer_pos = ctx.pointer_interact_pos();
        let new_window_pos = pointer_pos.unwrap_or_default();
        let new_window_size = Vec2::new(500., 500.);
        info!("Opening window at {new_window_pos}");
        app.egui_config.open_dirs.insert(
            work_dir,
            Rect::from_min_size(new_window_pos, new_window_size),
        );
    }
}

/// Drag-and-drop response data
struct DNDResponse {
    source_index: usize,
    destination_index: usize,
}

/// Returns the drag-and-drop destination index
fn ui_work_dir_row(
    ui: &mut Ui,
    app: &mut MyApp,
    index: usize,
    work_dir: &PathBuf,
) -> Option<DNDResponse> {
    let id = Id::new(work_dir);
    let response = ui
        .horizontal(|ui| {
            let rtn = ui
                .dnd_drag_source(id, index, |ui| {
                    ui.label(work_dir.display().to_string());
                })
                .response;
            if ui.button("-").clicked() {
                app.work_dirs_config.work_dirs.remove(work_dir);
                // will be auto-saved later
            }
            rtn
        })
        .inner;

    if let (Some(pointer), Some(source_index)) = (
        ui.input(|input_state| input_state.pointer.interact_pos()),
        response.dnd_hover_payload::<usize>(),
    ) {
        let rect = response.rect;

        // Preview insertion:
        let stroke = Stroke::new(1.0, Color32::WHITE);
        let destination_index = if *source_index == index {
            // We are dragged onto ourselves
            ui.painter().hline(rect.x_range(), rect.center().y, stroke);
            index
        } else if pointer.y < rect.center().y {
            // Above us
            ui.painter().hline(rect.x_range(), rect.top(), stroke);
            index
        } else {
            // Below us
            ui.painter().hline(rect.x_range(), rect.bottom(), stroke);
            index + 1
        };

        if let Some(source_index) = response.dnd_release_payload::<usize>() {
            // The user dropped onto this item.
            return Some(DNDResponse {
                source_index: *source_index,
                destination_index,
            });
        }
    }
    None
}
