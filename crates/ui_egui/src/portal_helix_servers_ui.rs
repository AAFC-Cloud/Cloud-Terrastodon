use crate::app::MyApp;
use crate::entra_directory_ui::loaded;
use crate::icons::HELIX_ICON;
use crate::portal_helix_server_row::PortalHelixServerRow;
use crate::reflected_table::ReflectedTable;
use crate::reflected_table_column::ReflectedTableColumn;
use cloud_terrastodon_command::app_work::Loadable;
use eframe::egui::Id;
use eframe::egui::Ui;
use std::collections::HashMap;
use teamy_tracing::DiagnosticReport;
use teamy_tracing::gui::DiagnosticReportView;

pub(crate) fn show_helix_servers(app: &mut MyApp, ui: &mut Ui) {
    if matches!(app.helix.servers, Loadable::NotLoaded) {
        app.refresh_helix_catalogue();
    }
    app.helix.show_catalogue_error(ui);
    if ui.button("Add server").clicked() {
        app.helix.server_form = Some(Default::default());
        app.helix.input_error = None;
    }
    if let Some(error) = &app.helix.input_error {
        ui.collapsing("Invalid server", |ui| {
            DiagnosticReportView::new(error).show(ui)
        });
    }
    if let Some(form) = &mut app.helix.server_form {
        match ui
            .add_enabled_ui(
                !matches!(app.helix.servers, Loadable::Loading { .. }),
                |ui| form.show(ui),
            )
            .inner
        {
            Ok(Some(server)) => {
                app.helix.server_form = None;
                app.helix.input_error = None;
                app.modify_helix_catalogue("Add Helix server", move |store| {
                    store.add(server).map(drop)
                });
            }
            Ok(None) => {}
            Err(error) => app.helix.input_error = Some(DiagnosticReport::new(error)),
        }
        if ui.button("Cancel").clicked() {
            app.helix.server_form = None;
            app.helix.input_error = None;
        }
    }
    let Some(catalogue) = loaded(ui, &app.helix.servers, "Could not load the saved servers.")
    else {
        return;
    };
    let rows: Vec<_> = catalogue
        .servers
        .iter()
        .map(|(name, url)| {
            PortalHelixServerRow::new(name, url, catalogue.default.as_ref() == Some(name))
        })
        .collect();
    let columns = [
        ReflectedTableColumn::new("Server", "name")
            .with_width(220.0)
            .with_icon(|_, _| Some(HELIX_ICON))
            .with_link("target"),
        ReflectedTableColumn::new("URL", "url").with_width(320.0),
        ReflectedTableColumn::new("Default", "default").with_width(100.0),
    ];
    if let Some(open) = ReflectedTable::new("helix-servers", &columns)
        .without_selection()
        .with_search_hint("Search servers")
        .with_result_count()
        .show_explorable(ui, &rows, |row| Id::new(&row.name), &mut HashMap::new())
    {
        open.apply(app);
    }
}
