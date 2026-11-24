use crate::app::MyApp;
use crate::portal_filter_chip::portal_filter_chip;
use crate::portal_search::portal_search;
use crate::reflected_table::ReflectedTable;
use crate::reflected_table_column::ReflectedTableColumn;
use crate::reflected_value::ReflectedValue;
use crate::scope_iam_thumbnails::load_queued_iam_thumbnails;
use crate::scope_iam_thumbnails::with_iam_thumbnails;
use crate::scope_role_assignment_filters::ScopeRoleAssignmentFilters;
use crate::scope_role_assignment_grouping::ScopeRoleAssignmentGrouping;
use crate::scope_role_assignment_row::ScopeRoleAssignmentRow;
use cloud_terrastodon_azure::EntraPrincipalId;
use eframe::egui::Id;
use eframe::egui::ScrollArea;
use eframe::egui::Ui;
use std::collections::HashSet;

pub(crate) fn show_role_assignments(
    app: &mut MyApp,
    ui: &mut Ui,
    id: Id,
    rows: &[ScopeRoleAssignmentRow],
) {
    let filters_key = id.with("filters");
    let mut filters: ScopeRoleAssignmentFilters = ui
        .data(|data| data.get_temp(filters_key))
        .unwrap_or_default();
    let previous_grouping = filters.grouping;
    filters.show(ui, rows);
    if filters.grouping != previous_grouping {
        ReflectedTable::new(id, &[]).reset_selection_anchor(ui);
    }
    let visible = filters.filtered(rows, app.egui_config.demo_mode);
    if visible.is_empty() {
        ui.label(if rows.is_empty() {
            "No role assignments were returned for this scope."
        } else {
            "No role assignments match the filters."
        });
    } else {
        show_assignment_table(app, ui, id, &visible, filters.grouping);
    }
    ui.data_mut(|data| data.insert_temp(filters_key, filters));
}

pub(crate) fn show_check_access(
    app: &mut MyApp,
    ui: &mut Ui,
    id: Id,
    rows: &[ScopeRoleAssignmentRow],
) {
    let selected_key = id.with("principal");
    let search_key = id.with("principal-search");
    let mut selected: Option<EntraPrincipalId> = ui
        .data(|data| data.get_temp(selected_key))
        .unwrap_or_default();
    let mut query: String = ui
        .data(|data| data.get_temp(search_key))
        .unwrap_or_default();
    let demo_mode = app.egui_config.demo_mode;
    let label = selected.map_or_else(
        || "Select principal".to_owned(),
        |principal| {
            rows.iter()
                .find(|row| row.assignment.principal_id == principal)
                .map_or_else(|| principal.to_string(), |row| row.principal_name.clone())
        },
    );
    portal_filter_chip(ui, label, |ui| {
        portal_search(
            ui,
            &mut query,
            "Find a principal or enter an object ID",
            310.0,
        );
        let filter = query.trim().to_lowercase();
        if let Ok(principal) = query.trim().parse::<EntraPrincipalId>()
            && ui.button("Check object ID").clicked()
        {
            selected = Some(principal);
            ui.close();
        }
        let mut seen = HashSet::new();
        let mut principals: Vec<_> = rows
            .iter()
            .filter(|row| {
                seen.insert(row.assignment.principal_id)
                    && (row.principal_name.to_lowercase().contains(&filter)
                        || row.principal_id.contains(&filter)
                        || (!demo_mode
                            && row
                                .principal_email
                                .as_ref()
                                .is_some_and(|email| email.to_lowercase().contains(&filter))))
            })
            .collect();
        principals.sort_by_cached_key(|row| {
            (
                row.principal_name.to_lowercase(),
                *row.assignment.principal_id.as_ref(),
            )
        });
        ScrollArea::vertical().max_height(260.0).show(ui, |ui| {
            for row in principals {
                if ui
                    .selectable_value(
                        &mut selected,
                        Some(row.assignment.principal_id),
                        &row.principal_name,
                    )
                    .on_hover_text(&row.principal_id)
                    .clicked()
                {
                    ui.close();
                }
            }
        });
    });
    ui.data_mut(|data| {
        data.insert_temp(selected_key, selected);
        data.insert_temp(search_key, query);
    });
    if let Some(principal) = selected {
        ui.label("Direct and inherited assignments to this principal.");
        let visible: Vec<_> = rows
            .iter()
            .filter(|row| row.assignment.principal_id == principal)
            .collect();
        if visible.is_empty() {
            ui.label("No visible role assignments for this principal.");
        } else {
            show_assignment_table(
                app,
                ui,
                id.with(principal),
                &visible,
                ScopeRoleAssignmentGrouping::Role,
            );
        }
    }
}

fn show_assignment_table(
    app: &mut MyApp,
    ui: &mut Ui,
    id: Id,
    rows: &[&ScopeRoleAssignmentRow],
    grouping: ScopeRoleAssignmentGrouping,
) {
    let columns = [
        ReflectedTableColumn::new("Name", "principal_name")
            .with_width(224.0)
            .with_icon(ScopeRoleAssignmentRow::icon)
            .with_secondary_text("principal_detail")
            .with_link("principal_link"),
        ReflectedTableColumn::new("Type", "principal_type").with_width(110.0),
        ReflectedTableColumn::new("Role", "role").with_width(140.0),
        ReflectedTableColumn::new("Scope", "scope_label")
            .with_width(158.0)
            .with_link("scope"),
        ReflectedTableColumn::new("Access", "inheritance").with_width(96.0),
        ReflectedTableColumn::new("Condition", "condition").with_width(110.0),
    ];
    let row_id = |row: &ScopeRoleAssignmentRow| id.with(&row.assignment.id);
    if let Some(row) = rows
        .iter()
        .find(|row| app.azure.checkboxes.get(&row_id(row)) == Some(&true))
    {
        ui.collapsing("Assignment details", |ui| {
            ReflectedValue::new(row.assignment.as_ref()).show(ui)
        });
    }
    let table = ReflectedTable::new(id, &columns);
    let target = with_iam_thumbnails(
        &app.azure.scope_iam.thumbnails,
        app.egui_config.demo_mode,
        ui,
        |ui| {
            if grouping == ScopeRoleAssignmentGrouping::None {
                table.show(ui, rows.iter().copied(), row_id, &mut app.azure.checkboxes)
            } else {
                table.show_grouped(
                    ui,
                    rows.iter().copied(),
                    row_id,
                    &mut app.azure.checkboxes,
                    |row| grouping.group(row),
                )
            }
        },
    );
    load_queued_iam_thumbnails(app, ui);
    if let Some(target) = target {
        target.apply(app);
    }
}
