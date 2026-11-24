use crate::app::MyApp;
use crate::azure_devops_project_data_ui::show_azure_devops_project_data;
use crate::azure_devops_project_data_ui::show_azure_devops_project_overview;
use crate::icons::DEVOPS_PROJECT_ICON;
use crate::portal_azure_devops_organization_view::PortalAzureDevOpsOrganizationView;
use crate::portal_azure_devops_project_view::PortalAzureDevOpsProjectView;
use crate::portal_tab_open::PortalTabOpen;
use crate::resource_detail_header::resource_detail_header;
use crate::scope_navigation::navigation_tile;
use crate::sidebar_expansion_toggle::sidebar_expansion_toggle;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectId;
use cloud_terrastodon_command::app_work::Loadable;
use eframe::egui::CentralPanel;
use eframe::egui::Color32;
use eframe::egui::Frame;
use eframe::egui::Margin;
use eframe::egui::Panel;
use eframe::egui::ScrollArea;
use eframe::egui::Ui;
use std::borrow::Cow;
use teamy_tracing::gui::DiagnosticReportView;

pub(crate) fn show_azure_devops_project(
    app: &mut MyApp,
    ui: &mut Ui,
    organization: &AzureDevOpsOrganizationUrl,
    project: AzureDevOpsProjectId,
    view: PortalAzureDevOpsProjectView,
) {
    let project_name = app
        .azure_devops_project_name_in(&app.azure, organization, project)
        .map_or_else(|| Cow::Owned(project.to_string()), Cow::Borrowed);
    let projects = PortalAzureDevOpsOrganizationView::Projects.target(organization);
    if resource_detail_header(
        ui,
        project_name.as_ref(),
        "Azure DevOps project",
        "Projects",
        DEVOPS_PROJECT_ICON,
        &projects,
    )
    .is_some_and(|open| open.apply(app))
    {
        return;
    }

    let expanded = app.resource_navigation_expanded;
    let open = Panel::left(ui.make_persistent_id("devops-project-navigation"))
        .exact_size(if expanded { 174.0 } else { 44.0 })
        .resizable(false)
        .frame(Frame::new().fill(Color32::WHITE))
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            sidebar_expansion_toggle(
                ui,
                &mut app.resource_navigation_expanded,
                "project navigation",
            );
            ScrollArea::vertical()
                .show(ui, |ui| {
                    let mut destination = None;
                    for section in PortalAzureDevOpsProjectView::ALL {
                        if let Some(open) = navigation_tile(
                            ui,
                            section.label(),
                            section.icon(),
                            section == view,
                            expanded,
                            || section.target(organization, project),
                        ) {
                            destination = Some(open);
                        }
                    }
                    destination
                })
                .inner
        })
        .inner;
    match open {
        Some(PortalTabOpen::Current(target)) => {
            app.tabs.replace_current(target);
            app.record_portal_visit();
            app.after_history_navigation();
            ui.ctx().request_repaint();
            return;
        }
        Some(PortalTabOpen::Background(target)) => app.open_background_tab(target),
        None => {}
    }

    CentralPanel::default()
        .frame(Frame::NONE.inner_margin(Margin::symmetric(16, 8)))
        .show(ui, |ui| {
            show_project_section(app, ui, organization, project, view)
        });
}

fn show_project_section(
    app: &mut MyApp,
    ui: &mut Ui,
    organization: &AzureDevOpsOrganizationUrl,
    project: AzureDevOpsProjectId,
    view: PortalAzureDevOpsProjectView,
) {
    app.show_azure_devops_credentials_for(ui, organization);
    let Some(authentication) = app.ready_azure_devops_authentication_for(organization) else {
        app.show_azure_devops_authentication_required(ui, organization);
        return;
    };
    if matches!(
        app.azure
            .azure_devops_projects
            .get(organization, authentication),
        Loadable::NotLoaded
    ) {
        app.load_azure_devops_projects_for(organization, false);
    }

    ui.push_id(
        (
            organization,
            project,
            view,
            authentication,
            app.azure.account_generation,
        ),
        |ui| {
            if view == PortalAzureDevOpsProjectView::Overview {
                match app
                    .azure
                    .azure_devops_projects
                    .get(organization, authentication)
                {
                    Loadable::NotLoaded | Loadable::Loading { .. } => {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label("Loading project…");
                        });
                    }
                    Loadable::Failed { error, .. } => {
                        ui.label("Could not load this project.");
                        ui.collapsing("Error details", |ui| {
                            DiagnosticReportView::new(error).show(ui)
                        });
                    }
                    Loadable::Loaded {
                        value: projects, ..
                    } => {
                        if let Some(project) = projects.iter().find(|entry| entry.id == project) {
                            show_azure_devops_project_overview(ui, project);
                        } else {
                            ui.label(
                                "This project is absent from the organization's project inventory.",
                            );
                        }
                    }
                }
                return;
            }

            if matches!(
                app.azure.azure_devops_projects.section(
                    organization,
                    authentication,
                    project,
                    view
                ),
                Loadable::NotLoaded
            ) {
                app.load_azure_devops_project_section(organization, project, view, false);
            }
            match app.azure.azure_devops_projects.section(
                organization,
                authentication,
                project,
                view,
            ) {
                Loadable::NotLoaded | Loadable::Loading { .. } => {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(format!("Loading {}…", view.label().to_lowercase()));
                    });
                }
                Loadable::Failed { error, .. } => {
                    ui.label(format!("Could not load {}.", view.label().to_lowercase()));
                    ui.collapsing("Error details", |ui| {
                        DiagnosticReportView::new(error).show(ui)
                    });
                }
                Loadable::Loaded { value, .. } => {
                    show_azure_devops_project_data(
                        ui,
                        organization,
                        project,
                        value,
                        &mut app.azure.checkboxes,
                    );
                }
            }
        },
    );
}
