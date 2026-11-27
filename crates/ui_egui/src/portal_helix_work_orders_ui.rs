use crate::app::MyApp;
use crate::icons::HELIX_WORK_ORDER_ICON;
use crate::portal_helix_server_view::PortalHelixServerView;
use crate::portal_helix_work::PortalHelixWork;
use crate::portal_helix_work_key::PortalHelixWorkKey;
use crate::portal_helix_work_order_row::PortalHelixWorkOrderRow;
use crate::reflected_table::ReflectedTable;
use crate::reflected_table_column::ReflectedTableColumn;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountId;
use cloud_terrastodon_bmc_helix_types::BmcHelixServer;
use cloud_terrastodon_command::app_work::Loadable;
use eframe::egui::CollapsingHeader;
use eframe::egui::Id;
use eframe::egui::Ui;
use std::collections::HashMap;
use teamy_tracing::DiagnosticReport;
use teamy_tracing::gui::DiagnosticReportView;

pub(crate) fn show_work_orders(
    app: &mut MyApp,
    ui: &mut Ui,
    server: &BmcHelixServer,
    account: CloudTerrastodonAccountId,
) {
    let tab = app.tabs.active_id();
    let view = PortalHelixServerView::WorkOrders;
    let work = app.helix.work(tab, server, account, &view);
    let key = PortalHelixWorkKey::new(tab, server, account, &view);
    let lookup = app.helix.work_order_lookup(&key);
    let matching_order = lookup.and_then(|lookup| lookup.matching_order());
    let orders = match work.as_loaded() {
        Some(PortalHelixWork::List(orders)) => orders.as_slice(),
        _ => &[],
    };
    let rows: Vec<_> = orders
        .iter()
        .filter(|order| matching_order.is_none_or(|matched| matched.id != order.id))
        .chain(matching_order)
        .map(|order| PortalHelixWorkOrderRow::new(&server.name, order))
        .collect();
    let status = if lookup.is_some_and(|lookup| lookup.is_searching()) {
        Some("Searching Helix…")
    } else if matches!(work, Loadable::Loading { .. }) {
        Some("Loading work orders…")
    } else {
        None
    };
    if let Loadable::Failed { error, .. } = work {
        show_failure(ui, "Could not load work orders", error);
    }
    if let Some(Loadable::Failed { error, .. }) = lookup.map(|lookup| &lookup.result) {
        show_failure(ui, "Could not search Helix for this work order", error);
    }

    let table_id = Id::new(("helix-work-orders", tab, &server.url, account));
    let columns = [
        ReflectedTableColumn::new("Work order", "id")
            .with_width(180.0)
            .with_icon(|_, _| Some(HELIX_WORK_ORDER_ICON))
            .with_link("target"),
        ReflectedTableColumn::new("Status", "status")
            .with_width(130.0)
            .groupable(),
        ReflectedTableColumn::new("Summary", "summary"),
    ];
    let table = ReflectedTable::new(table_id, &columns)
        .without_selection()
        .with_search_hint("Search work orders by ID or text")
        .with_status(status);
    if let Some(open) =
        table.show_explorable(ui, &rows, |row| table_id.with(&row.id), &mut HashMap::new())
    {
        open.apply(app);
        return;
    }
    app.search_helix_work_orders(server, account, &table.search_text(ui), ui.ctx());
}

fn show_failure(ui: &mut Ui, title: &str, error: &DiagnosticReport) {
    CollapsingHeader::new(title).show(ui, |ui| DiagnosticReportView::new(error).show(ui));
}
