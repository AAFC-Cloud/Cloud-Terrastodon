use crate::app::MyApp;
use crate::icons::page_icon;
use crate::pim_activation::open_pim_activation;
use crate::pim_activation::show_pim_activation;
use crate::pim_activation_assignment::PimActivationAssignment;
use crate::pim_assignment_status::PimAssignmentStatus;
use crate::pim_role_row::PimRoleRow;
use crate::portal_page::PortalPage;
use crate::portal_section_tabs::portal_section_tabs;
use crate::portal_tab_open::PortalTabOpen;
use crate::portal_tab_target::PortalTabTarget;
use crate::reflected_table::ReflectedTable;
use crate::reflected_table_column::ReflectedTableColumn;
use crate::scope_navigation::navigation_tile;
use crate::sidebar_expansion_toggle::sidebar_expansion_toggle;
use cloud_terrastodon_auth_types::AzureBearerAuthSource;
use cloud_terrastodon_command::app_work::Loadable;
use eframe::egui::CentralPanel;
use eframe::egui::Color32;
use eframe::egui::Frame;
use eframe::egui::Id;
use eframe::egui::Image;
use eframe::egui::Margin;
use eframe::egui::OpenUrl;
use eframe::egui::Panel;
use eframe::egui::RichText;
use eframe::egui::Ui;
use eframe::egui::Vec2;
use jiff::Timestamp;
use std::collections::HashMap;
use teamy_tracing::gui::DiagnosticReportView;

pub(crate) const PIM_PORTAL_URL: &str = "https://aka.ms/pim";

pub(crate) fn show_portal_pim(app: &mut MyApp, ui: &mut Ui, page: PortalPage) {
    Frame::NONE
        .inner_margin(Margin::symmetric(16, 10))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add(Image::new(page_icon(page)).fit_to_exact_size(Vec2::splat(30.0)));
                ui.label(RichText::new(page.title()).strong().size(22.0));
            });
        });

    let expanded = app.resource_navigation_expanded;
    let destination = Panel::left(ui.make_persistent_id("pim-navigation"))
        .exact_size(if expanded { 186.0 } else { 44.0 })
        .resizable(false)
        .frame(Frame::new().fill(Color32::WHITE))
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            sidebar_expansion_toggle(ui, &mut app.resource_navigation_expanded, "PIM navigation");
            [
                (PortalPage::Pim, "Quick start"),
                (PortalPage::PimEntraRoles, "Microsoft Entra roles"),
                (PortalPage::PimAzureResources, "Azure resources"),
            ]
            .into_iter()
            .filter_map(|(section, label)| {
                navigation_tile(
                    ui,
                    label,
                    page_icon(section),
                    page == section,
                    expanded,
                    || PortalTabTarget::Page(section),
                )
            })
            .last()
        })
        .inner;
    match destination {
        Some(PortalTabOpen::Current(target)) => {
            app.tabs.replace_current(target);
            app.record_portal_visit();
            app.after_history_navigation();
            ui.ctx().request_repaint();
            return;
        }
        Some(PortalTabOpen::Background(target)) => {
            app.open_background_tab(target);
        }
        None => {}
    }

    show_pim_activation(app, ui, page);
    CentralPanel::default()
        .frame(Frame::NONE.inner_margin(Margin::symmetric(16, 8)))
        .show(ui, |ui| {
            if page == PortalPage::Pim {
                show_pim_quick_start(app, ui);
            } else {
                show_pim_roles(app, ui, page);
            }
        });
}

fn show_pim_quick_start(app: &mut MyApp, ui: &mut Ui) {
    ui.heading("My roles");
    for (page, description) in [
        (
            PortalPage::PimEntraRoles,
            "Directory role assignments for the selected account.",
        ),
        (
            PortalPage::PimAzureResources,
            "Eligible roles on Azure resources and management scopes.",
        ),
    ] {
        ui.add_space(8.0);
        let response = ui.link(page.title());
        if PortalTabOpen::from_response(&response, || PortalTabTarget::Page(page))
            .is_some_and(|open| open.apply(app))
        {
            return;
        }
        ui.label(description);
    }
    ui.add_space(8.0);
    ui.hyperlink_to(
        "Manage groups and requests in Microsoft PIM",
        PIM_PORTAL_URL,
    );
}

fn show_pim_roles(app: &mut MyApp, ui: &mut Ui, page: PortalPage) {
    let Some(authentication) = app.ready_azure_authentication() else {
        app.show_azure_authentication_required(ui);
        return;
    };
    if authentication.auth_source == AzureBearerAuthSource::WorkloadIdentity {
        ui.label("Choose a Browser or Azure CLI account to view your PIM roles.");
        return;
    }
    app.load_pim_roles(page, false);
    let now = Timestamp::now();
    let id = ui.make_persistent_id((page.title(), app.azure.account_generation));
    let mut activation = None;
    let navigation = match page {
        PortalPage::PimEntraRoles => {
            let Some(assignments) = show_loaded_assignments(ui, &app.azure.pim_roles.entra) else {
                return;
            };
            let status = assignment_tabs(
                ui,
                id,
                true,
                assignments
                    .iter()
                    .map(|row| PimRoleRow::entra_status(row, now)),
            );
            let tenant = authentication.tenant_id.to_string();
            let rows = assignments
                .iter()
                .enumerate()
                .filter(|(_, item)| PimRoleRow::entra_status(item, now) == status)
                .map(|(index, item)| PimRoleRow::entra(index, item, &tenant))
                .collect::<Vec<_>>();
            let (open, chosen) =
                show_assignment_table(ui, id, &rows, false, status, &mut app.azure.checkboxes);
            if let Some(row) = chosen
                && let Some(item) = assignments.get(row.index)
                && PimActivationAssignment::can_activate_entra(&item.assignment)
            {
                activation = Some(PimActivationAssignment::Entra {
                    assignment: item.assignment.clone(),
                    role_name: row.role.clone(),
                });
            }
            open
        }
        PortalPage::PimAzureResources => {
            let Some(assignments) = show_loaded_assignments(ui, &app.azure.pim_roles.azure) else {
                return;
            };
            let status = assignment_tabs(
                ui,
                id,
                false,
                assignments
                    .iter()
                    .map(|row| PimRoleRow::azure_status(row, now)),
            );
            let rows = assignments
                .iter()
                .enumerate()
                .filter(|(_, item)| PimRoleRow::azure_status(item, now) == status)
                .map(|(index, item)| PimRoleRow::azure(index, item))
                .collect::<Vec<_>>();
            let (open, chosen) =
                show_assignment_table(ui, id, &rows, true, status, &mut app.azure.checkboxes);
            if let Some(row) = chosen
                && let Some(item) = assignments.get(row.index)
            {
                if PimActivationAssignment::can_activate_azure(item) {
                    activation = Some(PimActivationAssignment::Azure(item.clone()));
                } else if row.action == "Open in portal" {
                    ui.ctx().open_url(OpenUrl::new_tab(PIM_PORTAL_URL));
                }
            }
            open
        }
        _ => return,
    };
    if navigation.is_some_and(|open| open.apply(app)) {
        return;
    }
    if let Some(assignment) = activation {
        open_pim_activation(app, assignment);
        ui.ctx().request_repaint();
    }
}

fn show_loaded_assignments<'a, T>(ui: &mut Ui, inventory: &'a Loadable<T>) -> Option<&'a T> {
    match inventory {
        Loadable::Loaded { value, .. } => Some(value),
        Loadable::NotLoaded | Loadable::Loading { .. } => {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label("Loading assignments…");
            });
            None
        }
        Loadable::Failed { error, .. } => {
            ui.label("Could not load assignments.");
            ui.collapsing("Error details", |ui| {
                DiagnosticReportView::new(error).show(ui)
            });
            None
        }
    }
}

fn assignment_tabs(
    ui: &mut Ui,
    id: Id,
    shows_active_assignments: bool,
    received: impl Iterator<Item = PimAssignmentStatus>,
) -> PimAssignmentStatus {
    use PimAssignmentStatus::Active;
    use PimAssignmentStatus::Eligible;
    use PimAssignmentStatus::Expired;
    use PimAssignmentStatus::Scheduled;

    let available = received.collect::<Vec<_>>();
    let choices =
        [Eligible, Active, Expired, Scheduled]
            .into_iter()
            .filter(|status| match status {
                Eligible => true,
                Active => shows_active_assignments,
                Expired | Scheduled => available.contains(status),
            });
    let mut status = ui
        .data(|data| data.get_temp::<PimAssignmentStatus>(id))
        .unwrap_or_default();
    if !choices.clone().any(|choice| choice == status) {
        status = Eligible;
    }
    ui.horizontal_wrapped(|ui| {
        portal_section_tabs(ui, &mut status, choices, PimAssignmentStatus::label)
    });
    ui.data_mut(|data| data.insert_temp(id, status));
    status
}

fn show_assignment_table<'a>(
    ui: &mut Ui,
    id: Id,
    rows: &'a [PimRoleRow],
    azure_resources: bool,
    status: PimAssignmentStatus,
    checked_rows: &mut HashMap<Id, bool>,
) -> (Option<PortalTabOpen>, Option<&'a PimRoleRow>) {
    let mut columns = vec![
        ReflectedTableColumn::new("Role", "role"),
        ReflectedTableColumn::new("Scope", "scope")
            .with_link("scope_target")
            .groupable(),
    ];
    if azure_resources {
        columns.push(
            ReflectedTableColumn::new("Resource type", "scope_type")
                .with_width(135.0)
                .groupable(),
        );
        columns.push(ReflectedTableColumn::new("Condition", "condition").with_width(112.0));
    }
    columns.push(
        ReflectedTableColumn::new("Membership", "membership")
            .with_width(108.0)
            .groupable(),
    );
    columns.push(ReflectedTableColumn::new("End time", "end").with_width(164.0));
    if status == PimAssignmentStatus::Eligible {
        columns.push(
            ReflectedTableColumn::new("Action", "action")
                .with_width(117.0)
                .with_row_link(),
        );
    }
    let mut selected = None;
    let navigation = ReflectedTable::new(id.with(status.label()), &columns)
        .without_selection()
        .with_search_hint("Search by role or scope")
        .with_result_count()
        .show_explorable_with_activation(
            ui,
            rows,
            |row| id.with(&row.id),
            checked_rows,
            |row| {
                selected = Some(row.index);
            },
        );
    (
        navigation,
        selected.and_then(|index| rows.iter().find(|row| row.index == index)),
    )
}
