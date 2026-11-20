use crate::app::MyApp;
use crate::icons::DEVOPS_PROJECT_ICON;
use crate::inventory_table::show_inventory_table;
use crate::reflected_value::ReflectedValue;
use crate::table_row_background::TableRowBackground;
use crate::workers::load_azure_devops_projects::load_azure_devops_projects_with_invalidation;
use cloud_terrastodon_azure_devops::AzureDevOpsProject;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectId;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectVisibility;
use cloud_terrastodon_command::app_work::Loadable;
use eframe::egui::Button;
use eframe::egui::Checkbox;
use eframe::egui::Id;
use eframe::egui::Image;
use eframe::egui::ScrollArea;
use eframe::egui::Ui;
use eframe::egui::Vec2;
use egui_extras::Column;
use std::sync::Arc;
use teamy_tracing::gui::DiagnosticReportView;

/// Browse projects in the selected organization with its chosen credential.
pub(crate) fn azure_devops_projects_ui(app: &mut MyApp, ui: &mut Ui) {
    app.show_azure_devops_organization_picker(ui);
    if app.azure_devops_organization.is_none() || app.show_azure_devops_authentication_required(ui)
    {
        return;
    }

    let refresh = ui
        .add_enabled(
            !matches!(app.azure_devops_projects, Loadable::Loading { .. }),
            Button::new("Refresh"),
        )
        .clicked();
    if refresh || matches!(app.azure_devops_projects, Loadable::NotLoaded) {
        load_azure_devops_projects_with_invalidation(app, refresh);
    }

    match &app.azure_devops_projects {
        Loadable::NotLoaded | Loadable::Loading { .. } => {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label("Loading projects…");
            });
        }
        Loadable::Failed { error, .. } => {
            ui.label("Could not load projects.");
            ui.collapsing("Error details", |ui| {
                DiagnosticReportView::new(error).show(ui);
            });
        }
        Loadable::Loaded { value, .. } => {
            let projects = Arc::clone(value);
            let detail = ui.id().with((
                "project_details",
                app.account_generation,
                app.azure_devops_organization_picker.generation,
            ));
            let project_id = ui.data(|data| data.get_temp::<AzureDevOpsProjectId>(detail));
            if let Some(project) =
                project_id.and_then(|id| projects.iter().find(|project| project.id == id))
            {
                ui.horizontal_wrapped(|ui| {
                    if ui.link("All projects").clicked() {
                        ui.data_mut(|data| data.remove::<AzureDevOpsProjectId>(detail));
                    }
                    ui.heading(project.name.as_ref());
                });
                ui.push_id(project.id, |ui| {
                    ScrollArea::vertical()
                        .show(ui, |ui| ReflectedValue::new(project.as_ref()).show(ui));
                });
            } else if projects.is_empty() {
                ui.label("No projects are visible in this organization.");
            } else {
                show_projects_table(app, ui, &projects, detail);
            }
        }
    }
}

fn show_projects_table(
    app: &mut MyApp,
    ui: &mut Ui,
    projects: &[Arc<AzureDevOpsProject>],
    detail: Id,
) {
    let selected_count = projects
        .iter()
        .filter(|project| {
            app.checkboxes
                .get(&Id::new(project.id))
                .copied()
                .unwrap_or(false)
        })
        .count();
    show_inventory_table(ui, |table| {
        table
            .id_salt("azure_devops_projects")
            .column(Column::exact(32.0))
            .column(Column::remainder().at_least(140.0).clip(true))
            .column(Column::initial(106.0).at_least(75.0).clip(true))
            .column(Column::initial(125.0).at_least(100.0).clip(true))
            .header(27.0, |mut header| {
                header.col(|ui| {
                    let mut checked = selected_count == projects.len();
                    let indeterminate = selected_count > 0 && !checked;
                    if ui
                        .add(Checkbox::without_text(&mut checked).indeterminate(indeterminate))
                        .on_hover_text("Select all projects")
                        .changed()
                    {
                        for project in projects {
                            *app.checkbox_for(project.id) = checked;
                        }
                    }
                });
                for title in ["Name", "Visibility", "Last updated"] {
                    header.col(|ui| {
                        ui.strong(title);
                    });
                }
            })
            .body(|mut body| {
                let background = TableRowBackground::new(&mut body);
                body.rows(28.0, projects.len(), |mut row| {
                    let project = &projects[row.index()];
                    let checked = app.checkbox_for(project.id);
                    background.first_cell(&mut row, |ui| {
                        ui.add(Checkbox::without_text(checked))
                            .on_hover_text(format!("Select {}", project.name));
                        *checked
                    });
                    row.col(|ui| {
                        ui.add(
                            Image::new(DEVOPS_PROJECT_ICON).fit_to_exact_size(Vec2::splat(16.0)),
                        );
                        if ui.link(project.name.as_ref()).clicked() {
                            ui.data_mut(|data| data.insert_temp(detail, project.id));
                        }
                    });
                    row.col(|ui| {
                        ui.label(match project.visibility {
                            AzureDevOpsProjectVisibility::Private => "Private",
                            AzureDevOpsProjectVisibility::Public => "Public",
                        });
                    });
                    row.col(|ui| {
                        ui.label(project.last_update_time.format("%Y-%m-%d").to_string())
                            .on_hover_text(project.last_update_time.to_rfc3339());
                    });
                });
            });
    });
}
