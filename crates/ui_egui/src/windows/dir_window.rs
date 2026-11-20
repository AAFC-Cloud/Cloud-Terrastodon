use crate::app::MyApp;
use crate::local_path_preview::LocalPathPreview;
use crate::portal_tab_target::PortalTabTarget;
use cloud_terrastodon_command::app_work::Loadable;
use cloud_terrastodon_command::app_work::LoadableWorkBuilder;
use eframe::egui::ScrollArea;
use eframe::egui::Ui;
use std::path::Path;
use std::time::Duration;
use std::time::Instant;
use teamy_tracing::gui::DiagnosticReportView;

pub(crate) fn local_path_ui(app: &mut MyApp, ui: &mut Ui, path: &Path) {
    ui.horizontal_wrapped(|ui| {
        ui.monospace(path.display().to_string());
        if ui.button("Copy path").clicked() {
            ui.ctx().copy_text(path.display().to_string());
        }
    });

    if !app.local_path_previews.contains_key(path) {
        load_local_path(app, path);
    }

    let mut open = None;
    let mut retry = false;
    match app.local_path_previews.get(path) {
        Some(Loadable::Loaded { value, .. }) => match value {
            LocalPathPreview::Directory { entries, truncated } => {
                if entries.is_empty() {
                    ui.label("This directory is empty.");
                }
                ScrollArea::vertical().show(ui, |ui| {
                    for entry in entries {
                        let name = entry
                            .file_name()
                            .map_or_else(|| entry.to_string_lossy(), |name| name.to_string_lossy());
                        if ui.link(name.as_ref()).clicked() {
                            open = Some(entry.clone());
                        }
                    }
                    if *truncated {
                        ui.weak("Showing up to 256 entries.");
                    }
                });
            }
            LocalPathPreview::File {
                bytes,
                text,
                truncated,
            } => {
                ui.label(format!("{bytes} bytes"));
                if let Some(text) = text {
                    ScrollArea::both().show(ui, |ui| {
                        ui.monospace(text);
                    });
                }
                if *truncated {
                    ui.weak("Showing the first 64 KiB.");
                }
            }
            LocalPathPreview::Other => {
                ui.label("This path is neither a regular file nor a directory.");
            }
        },
        Some(Loadable::Failed { error, .. }) => {
            DiagnosticReportView::new(error).show(ui);
            retry = ui.button("Retry").clicked();
        }
        _ => {
            ui.spinner();
            ui.ctx().request_repaint_after(Duration::from_millis(100));
        }
    }

    if let Some(path) = open {
        app.navigate_tab(PortalTabTarget::LocalPath(path));
    }
    if retry {
        load_local_path(app, path);
    }
}

pub(crate) fn load_local_path(app: &mut MyApp, path: &Path) {
    let target = path.to_owned();
    let report_target = target.clone();
    let read_target = target.clone();
    let cancellation = app.executor.cancellation().clone();
    let started_at = Instant::now();
    let enqueue_result = LoadableWorkBuilder::<MyApp, _>::new()
        .description("Inspecting a local path")
        .setter(move |app, state| {
            let current_load = match &state {
                Loadable::Loading { .. } => true,
                Loadable::Loaded { started_at, .. } | Loadable::Failed { started_at, .. } => {
                    matches!(
                        app.local_path_previews.get(&report_target),
                        Some(Loadable::Loading { started_at: current }) if current == started_at
                    )
                }
                Loadable::NotLoaded => false,
            };
            if current_load && app.tabs.has_local_path(&report_target) {
                app.local_path_previews.insert(report_target.clone(), state);
            }
        })
        .work(async move |scope| LocalPathPreview::read(&read_target, &scope).await)
        .build()
        .enqueue(
            &app.work_tracker.clone(),
            app.tx.clone(),
            app,
            &cancellation,
        );
    if let Err(error) = enqueue_result {
        app.local_path_previews.insert(
            target,
            Loadable::Failed {
                error: error.into(),
                started_at,
                finished_at: Instant::now(),
            },
        );
    }
}
