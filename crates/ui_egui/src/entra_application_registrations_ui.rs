use crate::app::MyApp;
use crate::entra_application_credential_row::EntraApplicationCredentialRow;
use crate::entra_application_registration_row::EntraApplicationRegistrationRow;
use crate::entra_directory_ui::copyable_id;
use crate::entra_directory_ui::inventory_progress;
use crate::entra_directory_ui::loaded;
use crate::entra_directory_ui::show_inventory_errors;
use crate::icons::page_icon;
use crate::portal_page::PortalPage;
use crate::reflected_table::ReflectedTable;
use crate::reflected_table_column::ReflectedTableColumn;
use crate::reflected_value::ReflectedValue;
use cloud_terrastodon_azure::ArbitraryJson;
use cloud_terrastodon_azure::EntraApplicationObjectId;
use eframe::egui::Grid;
use eframe::egui::Id;
use eframe::egui::ImageSource;
use eframe::egui::ScrollArea;
use eframe::egui::Ui;
use jiff::Timestamp;
use std::sync::Arc;
use std::time::Duration;

pub(crate) fn show_app_registrations(app: &mut MyApp, ui: &mut Ui) {
    app.azure
        .entra_directory
        .app_registrations
        .receive_pages(|application| application.id);
    let inventory = &app.azure.entra_directory.app_registrations;
    show_inventory_errors(ui, inventory, "Could not load app registrations.");
    let progress = inventory_progress(inventory);

    let id = Id::new(("entra-app-registrations", app.azure.account_generation));
    let cache_id = ui.make_persistent_id("entra-app-registration-rows");
    let now = Timestamp::now();
    let minute = now.as_second().div_euclid(60);
    let snapshot = (app.azure.account_generation, inventory.revision, minute);
    let rows = ui
        .data(|data| {
            data.get_temp::<((u64, u64, i64), Arc<Vec<EntraApplicationRegistrationRow>>)>(cache_id)
        })
        .filter(|(key, _)| *key == snapshot)
        .map_or_else(
            || {
                let rows = Arc::new(
                    inventory
                        .rows(|application| application.id)
                        .map(|application| EntraApplicationRegistrationRow::new(application, now))
                        .collect(),
                );
                ui.data_mut(|data| data.insert_temp(cache_id, (snapshot, Arc::clone(&rows))));
                rows
            },
            |(_, rows)| rows,
        );
    // Recalculate credential validity at the next UTC minute or when a new
    // inventory arrives. Parsing Graph credential metadata stays off the frame loop.
    ui.ctx().request_repaint_after(Duration::from_secs(
        60 - now.as_second().rem_euclid(60) as u64,
    ));

    let primary = if app.egui_config.demo_mode {
        ReflectedTableColumn::new("Object ID", "id")
    } else {
        ReflectedTableColumn::new("Display name", "display_name")
    };
    let columns = [
        primary.with_link("id").with_icon(app_icon),
        ReflectedTableColumn::new("Application (client) ID", "client_id").with_width(268.0),
        ReflectedTableColumn::new("Created on", "created_on").with_width(102.0),
        ReflectedTableColumn::new("Certificates & secrets", "credential_status").with_width(158.0),
    ];
    let table = ReflectedTable::new(id, &columns)
        .with_status(progress.as_deref())
        .with_search_hint("Search app registrations by name or ID");
    let target = table.show_explorable_matching(
        ui,
        rows.iter(),
        |row| id.with(row.id),
        &mut app.azure.checkboxes,
        |row, search| !app.egui_config.demo_mode && inventory.lookup_matched(search, row.id),
    );
    app.search_entra_directory(
        PortalPage::EntraAppRegistrations,
        &table.search_text(ui),
        ui.ctx(),
    );
    if let Some(target) = target {
        target.apply(app);
    }
}

pub(crate) fn show_app_registration(app: &MyApp, ui: &mut Ui, id: EntraApplicationObjectId) {
    let Some(response) = app.azure.entra_directory.app_registration_details.get(&id) else {
        return;
    };
    let Some(application) = loaded(ui, response, "Could not load this app registration.") else {
        return;
    };

    ScrollArea::vertical()
        .id_salt(("entra-application", id))
        .show(ui, |ui| {
            Grid::new(("entra-application-details", id))
                .spacing([18.0, 6.0])
                .show(ui, |ui| {
                    copyable_id(ui, "Application (client) ID", application.app_id);
                    copyable_id(ui, "Object ID", application.id);
                    if let Some(authentication) = app.ready_azure_authentication() {
                        copyable_id(ui, "Directory (tenant) ID", authentication.tenant_id);
                    }
                    if let Some(created) = application.created_date_time {
                        property(ui, "Created on", &timestamp(created));
                    }
                    if let Some(audience) = &application.sign_in_audience {
                        property(ui, "Sign-in audience", audience);
                    }
                    if !app.egui_config.demo_mode {
                        if let Some(domain) = &application.publisher_domain {
                            property(ui, "Publisher domain", domain);
                        }
                        if let Some(description) = &application.description {
                            property(ui, "Description", description);
                        }
                        if !application.identifier_uris.is_empty() {
                            ui.label("Application ID URIs");
                            ui.vertical(|ui| {
                                for uri in &application.identifier_uris {
                                    ui.monospace(uri);
                                }
                            });
                            ui.end_row();
                        }
                    }
                });

            ui.add_space(14.0);
            ui.strong("Certificates & secrets");
            let now = Timestamp::now();
            show_credential_metadata(
                ui,
                "Certificates and keys",
                &application.key_credentials,
                now,
                app.egui_config.demo_mode,
            );
            show_credential_metadata(
                ui,
                "Client secrets",
                &application.password_credentials,
                now,
                app.egui_config.demo_mode,
            );
        });
}

fn app_icon(_: &Ui, _: ReflectedValue<'_>) -> Option<ImageSource<'static>> {
    Some(page_icon(PortalPage::EntraAppRegistrations))
}

fn property(ui: &mut Ui, name: &str, value: &str) {
    ui.label(name);
    ui.label(value);
    ui.end_row();
}

fn show_credential_metadata(
    ui: &mut Ui,
    title: &str,
    credentials: &[ArbitraryJson],
    now: Timestamp,
    demo_mode: bool,
) {
    ui.collapsing(format!("{title} ({})", credentials.len()), |ui| {
        if credentials.is_empty() {
            ui.label("None");
            return;
        }
        let columns = [
            ReflectedTableColumn::new("Name", "name"),
            ReflectedTableColumn::new("Status", "status").with_width(108.0),
            ReflectedTableColumn::new("Starts", "starts").with_width(150.0),
            ReflectedTableColumn::new("Expires", "expires").with_width(150.0),
        ];
        let id = Id::new(title);
        let rows: Vec<_> = credentials
            .iter()
            .enumerate()
            .map(|(index, credential)| {
                EntraApplicationCredentialRow::new(index, credential, now, demo_mode)
            })
            .collect();
        ReflectedTable::new(id, &columns)
            .without_selection()
            .with_search_hint("Search credentials")
            .show_explorable(ui, &rows, |row| id.with(row.index), &mut Default::default());
    });
}

fn timestamp(value: Timestamp) -> String {
    value.strftime("%Y-%m-%d %H:%M UTC").to_string()
}
