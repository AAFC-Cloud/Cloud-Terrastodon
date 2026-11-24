use crate::app::MyApp;
use crate::icons::DEVOPS_PROJECT_ICON;
use cloud_terrastodon_azure_devops::AzureDevOpsProject;
use cloud_terrastodon_command::app_work::Loadable;
use eframe::egui::Checkbox;
use eframe::egui::Id;
use eframe::egui::Ui;
use eframe::egui::Widget;
use eframe::egui::collapsing_header::CollapsingState;
use tracing::debug;

pub fn draw_devops_project_list_expando(app: &mut MyApp, ui: &mut Ui) {
    let mut expando = CollapsingState::load_with_default_open(
        ui.ctx(),
        ui.make_persistent_id("devops-projects-expando"),
        false,
    );
    let toggle_key = expando.id();
    if app.azure.toggle_intents.remove(&toggle_key) {
        expando.toggle(ui);
    }
    expando
        .clone()
        .show_header(ui, |ui| draw_expando_header(app, ui, toggle_key))
        .body(|ui| draw_expando_body(app, ui));
}

fn draw_expando_header(app: &mut MyApp, ui: &mut Ui, toggle_key: Id) {
    if ui.image(DEVOPS_PROJECT_ICON).clicked() {
        app.azure.toggle_intents.insert(toggle_key);
    }
    if let Some(organization) = &app.azure.azure_devops_organization
        && let Some(authentication) = app.ready_azure_devops_authentication_for(organization)
        && let Some(projects) = app
            .azure
            .azure_devops_projects
            .get(organization, authentication)
            .as_loaded()
    {
        let mut ids = projects
            .iter()
            .map(|project| Id::new((organization, authentication, project.id)));
        let mut all = ids
            .clone()
            .all(|id| app.azure.checkboxes.get(&id) == Some(&true));
        let any = ids.any(|id| app.azure.checkboxes.get(&id) == Some(&true));
        let indeterminate = any && !all;
        let elem = Checkbox::new(&mut all, "Azure DevOps Projects")
            .indeterminate(indeterminate)
            .ui(ui);
        if elem.changed() {
            for project in projects.iter() {
                app.azure
                    .checkboxes
                    .insert(Id::new((organization, authentication, project.id)), all);
            }
        }
    } else if ui.label("Azure DevOps Projects").clicked() {
        app.azure.toggle_intents.insert(toggle_key);
    }
}

fn draw_expando_body(app: &mut MyApp, ui: &mut Ui) {
    app.show_azure_devops_organization_picker(ui);
    let Some(organization) = app.azure.azure_devops_organization.clone() else {
        return;
    };
    let Some(authentication) = app.ready_azure_devops_authentication_for(&organization) else {
        app.show_azure_devops_authentication_required(ui, &organization);
        return;
    };
    if matches!(
        app.azure
            .azure_devops_projects
            .get(&organization, authentication),
        Loadable::NotLoaded
    ) {
        app.load_azure_devops_projects_for(&organization, false);
    }
    ui.vertical(|ui| {
        match app
            .azure
            .azure_devops_projects
            .get(&organization, authentication)
        {
            Loadable::NotLoaded => {
                ui.label("Not loaded");
            }
            Loadable::Loading { .. } => {
                ui.label("Loading...");
            }
            Loadable::Loaded {
                value: projects, ..
            } => {
                for project in projects.iter() {
                    draw_entry(
                        ui,
                        project,
                        app.azure
                            .checkboxes
                            .entry(Id::new((&organization, authentication, project.id)))
                            .or_default(),
                    );
                }
            }
            Loadable::Failed { error: err, .. } => {
                ui.label(format!("Error: {err}"));
                if ui.button("Retry").clicked() {
                    app.load_azure_devops_projects_for(&organization, false);
                }
            }
        }
    });
}

fn draw_entry(ui: &mut Ui, project: &AzureDevOpsProject, checked: &mut bool) {
    ui.horizontal(|ui| {
        let label = format!("{}", project.name);
        if ui.image(DEVOPS_PROJECT_ICON).clicked() {
            debug!("Clicked on Azure DevOps Project icon");
            *checked ^= true;
        }

        ui.checkbox(checked, label);
    });
}
