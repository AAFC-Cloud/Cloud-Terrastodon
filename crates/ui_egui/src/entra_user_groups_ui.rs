use crate::app::MyApp;
use crate::entra_directory_ui::loaded;
use crate::icons::page_icon;
use crate::portal_page::PortalPage;
use crate::portal_tab_open::PortalTabOpen;
use crate::reflected_table::ReflectedTable;
use crate::reflected_table_column::ReflectedTableColumn;
use crate::reflected_value::ReflectedValue;
use cloud_terrastodon_azure::EntraGroupId;
use cloud_terrastodon_azure::EntraUserId;
use eframe::egui::Id;
use eframe::egui::ImageSource;
use eframe::egui::Ui;
use facet::Facet;
use std::collections::HashMap;
use std::sync::Arc;

pub(crate) fn show_entra_user_group_memberships(
    app: &mut MyApp,
    ui: &mut Ui,
    user_id: EntraUserId,
) {
    app.load_entra_user_group_memberships(user_id, false);
    let Some(rows) = app
        .azure
        .entra_user_groups
        .memberships
        .get(&user_id)
        .and_then(|response| loaded(ui, response, "Could not load group memberships."))
    else {
        return;
    };
    ui.weak("Direct and indirect memberships");
    let id = ui.make_persistent_id(("group-memberships", user_id));
    if let Some(open) = show_group_table(
        ui,
        rows.iter().map(Arc::as_ref),
        |row| row.id,
        &mut app.azure.checkboxes,
        app.egui_config.demo_mode,
        id,
    ) {
        open.apply(app);
    }
}

pub(crate) fn show_entra_user_group_ownerships(app: &mut MyApp, ui: &mut Ui, user_id: EntraUserId) {
    app.load_entra_user_group_ownerships(user_id, false);
    let Some(rows) = app
        .azure
        .entra_user_groups
        .ownerships
        .get(&user_id)
        .and_then(|response| loaded(ui, response, "Could not load group ownerships."))
    else {
        return;
    };
    let id = ui.make_persistent_id(("group-ownerships", user_id));
    if let Some(open) = show_group_table(
        ui,
        rows.iter().map(Arc::as_ref),
        |row| row.id,
        &mut app.azure.checkboxes,
        app.egui_config.demo_mode,
        id,
    ) {
        open.apply(app);
    }
}

fn show_group_table<'row, T: Facet<'static> + 'row>(
    ui: &mut Ui,
    rows: impl IntoIterator<Item = &'row T>,
    row_id: impl Fn(&T) -> EntraGroupId,
    checked_rows: &mut HashMap<Id, bool>,
    demo_mode: bool,
    id: Id,
) -> Option<PortalTabOpen> {
    let object_id = ReflectedTableColumn::new("Object ID", "id").with_link("id");
    let columns: &[ReflectedTableColumn] = if demo_mode {
        &[object_id.with_icon(group_icon)]
    } else {
        &[
            ReflectedTableColumn::new("Display name", "display_name")
                .with_width(260.0)
                .with_link("id")
                .with_icon(group_icon),
            object_id,
        ]
    };
    ReflectedTable::new(id, columns)
        .with_search_hint("Search groups by name or ID")
        .with_result_count()
        .show_explorable(ui, rows, |row| id.with(row_id(row)), checked_rows)
}

fn group_icon(_: &Ui, _: ReflectedValue<'_>) -> Option<ImageSource<'static>> {
    Some(page_icon(PortalPage::EntraGroups))
}
