use crate::portal_filter_chip::portal_filter_chip;
use crate::portal_search::portal_search;
use crate::scope_role_assignment_grouping::ScopeRoleAssignmentGrouping;
use crate::scope_role_assignment_row::ScopeRoleAssignmentRow;
use crate::scope_role_assignment_scope_filter::ScopeRoleAssignmentScopeFilter;
use cloud_terrastodon_azure::HasSlug;
use cloud_terrastodon_azure::RoleDefinitionName;
use eframe::egui::Ui;
use std::collections::BTreeSet;

/// The remembered search, filters and grouping for one resource's role assignments.
#[derive(Clone, Default)]
pub(crate) struct ScopeRoleAssignmentFilters {
    search: String,
    principal_type: Option<String>,
    role: Option<RoleDefinitionName>,
    scope: ScopeRoleAssignmentScopeFilter,
    pub grouping: ScopeRoleAssignmentGrouping,
}

impl ScopeRoleAssignmentFilters {
    pub fn show(&mut self, ui: &mut Ui, rows: &[ScopeRoleAssignmentRow]) {
        ui.horizontal_wrapped(|ui| {
            portal_search(ui, &mut self.search, "Search assignments", 290.0);
            let principal_type = self.principal_type.as_deref().unwrap_or("All");
            portal_filter_chip(ui, format!("Type · {principal_type}"), |ui| {
                if ui
                    .selectable_value(&mut self.principal_type, None, "All types")
                    .clicked()
                {
                    ui.close();
                }
                let types: BTreeSet<_> =
                    rows.iter().map(|row| row.principal_type.as_str()).collect();
                for choice in types {
                    if ui
                        .selectable_value(&mut self.principal_type, Some(choice.to_owned()), choice)
                        .clicked()
                    {
                        ui.close();
                    }
                }
            });
            let role_label = self.role.map_or("All", |id| {
                rows.iter()
                    .find(|row| *row.assignment.role_definition_id.name() == id)
                    .map_or("Unavailable role", |row| row.role.as_str())
            });
            portal_filter_chip(ui, format!("Role · {role_label}"), |ui| {
                if ui
                    .selectable_value(&mut self.role, None, "All roles")
                    .clicked()
                {
                    ui.close();
                }
                let roles: BTreeSet<_> = rows
                    .iter()
                    .map(|row| (row.role.as_str(), *row.assignment.role_definition_id.name()))
                    .collect();
                for (label, id) in roles {
                    if ui
                        .selectable_value(&mut self.role, Some(id), label)
                        .on_hover_text(id.to_string())
                        .clicked()
                    {
                        ui.close();
                    }
                }
            });
            portal_filter_chip(ui, format!("Scope · {}", self.scope.label()), |ui| {
                for choice in ScopeRoleAssignmentScopeFilter::ALL {
                    if ui
                        .selectable_value(&mut self.scope, choice, choice.label())
                        .clicked()
                    {
                        ui.close();
                    }
                }
            });
            portal_filter_chip(ui, format!("Group by · {}", self.grouping.label()), |ui| {
                for choice in ScopeRoleAssignmentGrouping::ALL {
                    if ui
                        .selectable_value(&mut self.grouping, choice, choice.label())
                        .clicked()
                    {
                        ui.close();
                    }
                }
            });
        });
    }

    pub fn filtered<'a>(
        &self,
        rows: &'a [ScopeRoleAssignmentRow],
        demo_mode: bool,
    ) -> Vec<&'a ScopeRoleAssignmentRow> {
        let search = self.search.trim().to_lowercase();
        rows.iter()
            .filter(|row| {
                self.principal_type
                    .as_ref()
                    .is_none_or(|kind| kind == &row.principal_type)
            })
            .filter(|row| {
                self.role
                    .as_ref()
                    .is_none_or(|role| role == row.assignment.role_definition_id.name())
            })
            .filter(|row| {
                self.scope == ScopeRoleAssignmentScopeFilter::All
                    || self.scope == ScopeRoleAssignmentScopeFilter::of(row)
            })
            .filter(|row| search.is_empty() || row.matches(&search, demo_mode))
            .collect()
    }
}
