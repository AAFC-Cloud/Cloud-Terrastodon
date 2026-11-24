use crate::app::MyApp;
use crate::portal_resource_scope::PortalResourceScope;
use crate::reflected_table::ReflectedTable;
use crate::reflected_table_column::ReflectedTableColumn;
use crate::scope_activity_filters::ScopeActivityFilters;
use cloud_terrastodon_azure::AzureActivityLogEvent;
use cloud_terrastodon_azure::AzureActivityLogScope;
use eframe::egui::Id;
use eframe::egui::Ui;
use std::sync::Arc;

pub(crate) fn show_activity_log_results(
    app: &mut MyApp,
    ui: &mut Ui,
    origin: &PortalResourceScope,
    requested_scope: &AzureActivityLogScope,
    events: &[Arc<AzureActivityLogEvent>],
) {
    let id = Id::new(("activity-log", origin, app.azure.account_generation));
    let filter_id = ui.make_persistent_id(id.with("result-filters"));
    let mut filters: ScopeActivityFilters =
        ui.data(|data| data.get_temp(filter_id)).unwrap_or_default();
    ui.label("Filter returned events");
    filters.show(app, ui, id, requested_scope, events);

    let mut matching = filters.matching(events, requested_scope).peekable();
    if matching.peek().is_none() {
        ui.label(if events.is_empty() {
            "Azure returned no events for the requested scope and time range."
        } else {
            "No returned events match the result filters."
        });
    }
    let columns = [
        ReflectedTableColumn::new("Time (UTC)", "event_timestamp").with_width(180.0),
        ReflectedTableColumn::new("Operation", "operation_name.value")
            .with_width(230.0)
            .groupable(),
        ReflectedTableColumn::new("Status", "status.value")
            .with_width(112.0)
            .groupable(),
        ReflectedTableColumn::new("Severity", "level")
            .with_width(110.0)
            .groupable(),
        ReflectedTableColumn::new("Category", "category.value")
            .with_width(120.0)
            .groupable(),
        ReflectedTableColumn::new("Subscription", "subscription_id")
            .with_width(168.0)
            .with_link("subscription_id")
            .groupable(),
        ReflectedTableColumn::new("Resource", "resource_id")
            .with_link("resource_id")
            .groupable(),
    ];
    if let Some(target) = ReflectedTable::new(id, &columns)
        .initially_descending()
        .show_explorable(
            ui,
            matching,
            |event| id.with(&event.event_data_id),
            &mut app.azure.checkboxes,
        )
    {
        target.apply(app);
    }
    ui.data_mut(|data| data.insert_temp(filter_id, filters));
}
