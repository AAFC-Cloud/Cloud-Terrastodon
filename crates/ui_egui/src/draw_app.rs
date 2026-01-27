use crate::app::MyApp;
use crate::file_drag_and_drop::ui_file_drag_and_drop;
use crate::windows::dir_window::ui_dir_windows;
use crate::windows::selected_items_window::draw_selected_items_window;
use crate::windows::starting_points_window::draw_starting_points_window;
use crate::windows::work_dirs_window::draw_work_dirs_window;
use tracing::Level;
use egui_toast::Toast;
use egui_toast::ToastKind;
use egui_toast::ToastOptions;
impl MyApp {
    pub fn draw_app(&mut self, ctx: &eframe::egui::Context) {
        let app = self;

        // Top menu bar with About and Logs
        eframe::egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            eframe::egui::MenuBar::new().ui(ui, |ui| {
                // Logs toggle button
                if ui
                    .button(if app.logs_visible { "Logs (on)" } else { "Logs" })
                    .clicked()
                {
                    app.logs_visible = !app.logs_visible;
                }

                // About button
                if ui.button("About").clicked() {
                    app.about_open = !app.about_open;
                }

                // Right-align Quit (if desired) placeholder
                ui.with_layout(eframe::egui::Layout::right_to_left(eframe::egui::Align::Center), |ui| {
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(eframe::egui::ViewportCommand::Close);
                    }
                });
            });
        });

        // Draw existing windows
        draw_starting_points_window(app, ctx);
        draw_selected_items_window(app, ctx);
        draw_work_dirs_window(app, ctx);
        ui_file_drag_and_drop(app, ctx);
        ui_dir_windows(app, ctx);

        // About window
        if app.about_open {
            eframe::egui::Window::new("About")
                .resizable(false)
                .collapsible(false)
                .anchor(eframe::egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .open(&mut app.about_open)
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("cloud_terrastodon_ui_egui");
                        ui.add_space(10.0);
                        ui.label(format!("{}", app.app_info));
                        ui.add_space(10.0);
                        ui.label("UI for the Cloud Terrastodon project.");
                        ui.add_space(10.0);
                        ui.hyperlink_to("GitHub", "https://github.com/TeamDman/cloud-terrastodon");
                    });
                });
        }

        // Logs window (egui_tracing widget)
        if app.logs_visible {
            eframe::egui::Window::new("Logs")
                .default_size([800.0, 400.0])
                .open(&mut app.logs_visible)
                .show(ctx, |ui| {
                    let collector = cloud_terrastodon_tracing::event_collector();
                    ui.add(egui_tracing::Logs::new(collector));
                });
        }

        // Process new tracing events and create toasts for INFO and ERROR levels
        let collector = cloud_terrastodon_tracing::event_collector();
        let events = collector.events();
        let new_events = &events[app.last_seen_event_count..];
        for event in new_events {
            let kind = match event.level {
                Level::INFO => Some(ToastKind::Info),
                Level::ERROR => Some(ToastKind::Error),
                _ => None,
            };
            if let Some(kind) = kind {
                let message = event
                    .fields
                    .get("message")
                    .map_or("", std::string::String::as_str)
                    .to_string();
                app.toasts.add(
                    Toast::default()
                        .kind(kind)
                        .text(message)
                        .options(
                            ToastOptions::default()
                                .duration_in_seconds(5.0)
                                .show_progress(true)
                                .show_icon(true),
                        ),
                );
            }
        }
        app.last_seen_event_count = events.len();

        // Show toasts
        app.toasts.show(ctx);
    }
}
