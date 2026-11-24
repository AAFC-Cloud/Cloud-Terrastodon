use crate::app::MyApp;
use crate::portal_tab_open::PortalTabOpen;
use crate::portal_tab_target::PortalTabTarget;
use eframe::egui::Color32;
use eframe::egui::Context;
use eframe::egui::DragAndDrop;
use eframe::egui::Stroke;
use eframe::egui::Ui;
use eframe::egui::Window;
use std::path::PathBuf;

pub fn draw_work_dirs_window(app: &mut MyApp, ctx: &Context) {
    // Keep the existing floating-window behavior for backwards compatibility
    let mut dnd_response = None;

    Window::new("TF Work Dirs").show(ctx, |ui| {
        work_dirs_ui(app, ui, &mut dnd_response);
    });

    // handle drag and drop logic
    if let Some(DNDResponse {
        source_index,
        mut destination_index,
    }) = dnd_response
    {
        destination_index -= (source_index < destination_index) as usize;
        let Some(item) = app.work_dirs_config.work_dirs.remove_index(source_index) else {
            tracing::warn!(source_index, "Dragged work directory no longer exists");
            return;
        };
        destination_index = destination_index.min(app.work_dirs_config.work_dirs.len());
        app.work_dirs_config
            .work_dirs
            .shift_insert(destination_index, item);
    } else if DragAndDrop::has_payload_of_type::<usize>(ctx)
        && ctx.input(|input_state| input_state.pointer.any_released())
        && let Some(source_index) = DragAndDrop::take_payload::<usize>(ctx)
    {
        let Some(work_dir) = app
            .work_dirs_config
            .work_dirs
            .get_index(*source_index)
            .cloned()
        else {
            tracing::warn!(
                source_index = *source_index,
                "Dragged work directory no longer exists"
            );
            return;
        };
        app.navigate_tab(PortalTabTarget::LocalPath(work_dir));
    }
}

/// Tile-friendly body for the Work Dirs pane
pub fn work_dirs_ui(app: &mut MyApp, ui: &mut Ui, dnd_response: &mut Option<DNDResponse>) {
    use eframe::egui::Frame;

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
                        *dnd_response = Some(response);
                    }
                }
            });
        }
    });
    ui.visuals_mut().widgets.inactive.bg_fill = old;
    if let Some(source_index) = dropped_payload {
        // Dropped onto the area but not on any one item
        *dnd_response = Some(DNDResponse {
            source_index: *source_index,
            destination_index: app.work_dirs_config.work_dirs.len(),
        });
    }
}

/// Drag-and-drop response data
pub struct DNDResponse {
    pub source_index: usize,
    pub destination_index: usize,
}

/// Returns the drag-and-drop destination index
fn ui_work_dir_row(
    ui: &mut Ui,
    app: &mut MyApp,
    index: usize,
    work_dir: &PathBuf,
) -> Option<DNDResponse> {
    let id = ui.make_persistent_id(("work-directory-drag", work_dir));
    let response = ui
        .horizontal(|ui| {
            let rtn = ui
                .dnd_drag_source(id, index, |ui| {
                    let response = ui.link(work_dir.display().to_string());
                    if let Some(action) = PortalTabOpen::from_response(&response, || {
                        PortalTabTarget::LocalPath(work_dir.clone())
                    }) {
                        action.apply(app);
                    }
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
        let stroke = Stroke::new(1.0_f32, Color32::WHITE);
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
