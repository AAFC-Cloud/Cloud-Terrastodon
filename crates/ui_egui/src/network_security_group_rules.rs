use crate::app::MyApp;
use crate::network_security_group_rule_rows::NetworkSecurityGroupRuleRows;
use crate::network_security_rule_row::NetworkSecurityRuleRow;
use crate::portal_filter_chip::portal_filter_chip;
use crate::portal_search::portal_search;
use crate::reflected_table::ReflectedTable;
use crate::reflected_table_column::ReflectedTableColumn;
use crate::reflected_table_group::ReflectedTableGroup;
use cloud_terrastodon_azure::AzureNetworkSecurityGroup;
use eframe::egui::Id;
use eframe::egui::ScrollArea;
use eframe::egui::Ui;
use std::collections::BTreeSet;
use std::slice;
use std::sync::Arc;
use std::sync::Weak;

type RuleFilter = (&'static str, fn(&NetworkSecurityRuleRow) -> &[String]);
type CachedRuleRows = (
    Weak<AzureNetworkSecurityGroup>,
    Arc<NetworkSecurityGroupRuleRows>,
);

const FILTERS: [RuleFilter; 5] = [
    ("Port", |row| &row.ports),
    ("Protocol", |row| slice::from_ref(&row.protocol)),
    ("Source", |row| &row.sources),
    ("Destination", |row| &row.destinations),
    ("Action", |row| slice::from_ref(&row.action)),
];

const COLUMNS: [ReflectedTableColumn; 7] = [
    ReflectedTableColumn::new("Priority", "priority").with_width(85.0),
    ReflectedTableColumn::new("Name", "name"),
    ReflectedTableColumn::new("Port", "port").with_width(96.0),
    ReflectedTableColumn::new("Protocol", "protocol").with_width(96.0),
    ReflectedTableColumn::new("Source", "source").with_width(120.0),
    ReflectedTableColumn::new("Destination", "destination").with_width(120.0),
    ReflectedTableColumn::new("Action", "action").with_width(96.0),
];

pub(crate) fn network_security_group_rules_ui(
    app: &mut MyApp,
    ui: &mut Ui,
    group: &Arc<AzureNetworkSecurityGroup>,
) {
    let view = ui.make_persistent_id((
        "network-security-group-rules",
        app.azure.account_generation,
        &group.id,
    ));
    let rows_id = view.with("rows");
    let cached = ui.data(|data| data.get_temp::<CachedRuleRows>(rows_id));
    let rows = if let Some((previous, rows)) = cached
        && std::ptr::eq(previous.as_ptr(), Arc::as_ptr(group))
    {
        rows
    } else {
        let rows = Arc::new(NetworkSecurityGroupRuleRows::from(&group.properties));
        ui.data_mut(|data| data.insert_temp(rows_id, (Arc::downgrade(group), Arc::clone(&rows))));
        rows
    };
    match (rows.custom_available, rows.defaults_available) {
        (false, false) => {
            ui.label("Azure did not return security rules for this group.");
            return;
        }
        (false, true) => {
            ui.weak("Azure did not return custom security rules.");
        }
        (true, false) => {
            ui.weak("Azure did not return default security rules.");
        }
        (true, true) => {}
    };
    if rows.rows.is_empty() {
        if rows.custom_available && rows.defaults_available {
            ui.label("No security rules.");
        }
        return;
    }

    let search_id = view.with("search");
    let mut search = ui.data(|data| data.get_temp::<String>(search_id).unwrap_or_default());
    let selections = ui
        .horizontal_wrapped(|ui| {
            if portal_search(ui, &mut search, "Filter by name", 240.0).changed() {
                ui.data_mut(|data| data.insert_temp(search_id, search.clone()));
            }
            FILTERS.map(|filter| rule_filter(ui, view.with(filter.0), filter, &rows.rows))
        })
        .inner;
    let search = search.trim().to_lowercase();
    let matching = rows.rows.iter().filter(|row| {
        (search.is_empty() || row.name.to_lowercase().contains(&search))
            && FILTERS
                .iter()
                .zip(&selections)
                .all(|((_, values), selected)| {
                    selected.is_empty() || values(row).iter().any(|value| selected.contains(value))
                })
    });
    let mut matching = matching.peekable();
    if matching.peek().is_none() {
        ui.label("No security rules match the current filters.");
        return;
    }
    if let Some(destination) = ReflectedTable::new(view, &COLUMNS).show_grouped(
        ui,
        matching,
        |row| view.with(&row.id),
        &mut app.azure.checkboxes,
        |row| ReflectedTableGroup {
            id: Id::new(&row.direction),
            label: format!("{} security rules", row.direction.as_str()),
        },
    ) {
        destination.apply(app);
    }
}

fn rule_filter(
    ui: &mut Ui,
    id: Id,
    (label, values): RuleFilter,
    rows: &[NetworkSecurityRuleRow],
) -> BTreeSet<String> {
    let mut selected = ui.data(|data| data.get_temp::<BTreeSet<String>>(id).unwrap_or_default());
    let display = match selected.len() {
        0 => format!("{label} · All"),
        count => format!("{label} · {count} selected"),
    };
    let mut changed = false;
    ui.push_id(id, |ui| {
        portal_filter_chip(ui, display, |ui| {
            ui.set_min_width(190.0);
            let search_id = id.with("search");
            let mut search = ui.data(|data| data.get_temp::<String>(search_id).unwrap_or_default());
            if portal_search(ui, &mut search, "Filter values", 190.0).changed() {
                ui.data_mut(|data| data.insert_temp(search_id, search.clone()));
            }
            let search = search.trim().to_lowercase();
            if ui.selectable_label(selected.is_empty(), "All").clicked() {
                selected.clear();
                changed = true;
            }
            ScrollArea::vertical()
                .id_salt(id)
                .max_height(220.0)
                .show(ui, |ui| {
                    let choices: BTreeSet<_> = rows
                        .iter()
                        .flat_map(values)
                        .map(String::as_str)
                        .filter(|value| value.to_lowercase().contains(&search))
                        .collect();
                    for value in choices {
                        let mut checked = selected.contains(value);
                        if ui.checkbox(&mut checked, value).changed() {
                            if checked {
                                selected.insert(value.to_owned());
                            } else {
                                selected.remove(value);
                            }
                            changed = true;
                        }
                    }
                });
        });
    });
    if changed {
        ui.data_mut(|data| data.insert_temp(id, selected.clone()));
    }
    selected
}
