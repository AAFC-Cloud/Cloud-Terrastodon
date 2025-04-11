use crate::app::MyApp;
use crate::icons::DEVOPS_ICON;
use crate::loadable::Loadable;
use crate::workers::load_azure_devops_projects::load_azure_devops_projects;
use cloud_terrastodon_core_azure_devops::prelude::AzureDevOpsProject;
use eframe::egui;
use eframe::egui::Checkbox;
use eframe::egui::Context;
use eframe::egui::Id;
use eframe::egui::Ui;
use eframe::egui::Widget;
use eframe::egui::collapsing_header::CollapsingState;
use tracing::debug;

pub fn draw_devops_project_list_expando(app: &mut MyApp, ctx: &Context, ui: &mut Ui) {
    let mut expando =
        CollapsingState::load_with_default_open(ctx, "azure devops projects".into(), false);
    let toggle_key = expando.id();
    if app.toggle_intents.remove(&toggle_key) {
        expando.toggle(&ui);
    }
    let is_open = expando.is_open();
    if is_open && matches!(app.azure_devops_projects, Loadable::NotLoaded) {
        load_azure_devops_projects(app);
    }
    expando
        .clone()
        .show_header(ui, |ui| draw_expando_header(app, ui, toggle_key))
        .body(|ui| draw_expando_body(app, ctx, ui));
}

fn draw_expando_header(app: &mut MyApp, ui: &mut Ui, toggle_key: Id) {
    match &app.azure_devops_projects {
        Loadable::Loaded(projects) => {
            let projects = projects.clone();
            if ui.image(DEVOPS_ICON).clicked() {
                app.toggle_intents.insert(toggle_key);
            }

            let mut all = projects.iter().all(|sub| *app.checkbox_for(&sub.id));
            let any = projects.iter().any(|sub| *app.checkbox_for(&sub.id));
            let indeterminate = any && !all;
            let elem = Checkbox::new(&mut all, "Azure DevOps Projects")
                .indeterminate(indeterminate)
                .ui(ui);
            if elem.changed() {
                for project in projects.iter() {
                    *app.checkbox_for(&project.id) = all
                }
            }
        }
        _ => {
            if ui.image(DEVOPS_ICON).clicked() {
                debug!("Clicked on Azure DevOps Projects header icon");
                app.toggle_intents.insert(toggle_key);
            }
            let elem = ui.label("Azure DevOps Projects");
            if elem.clicked() {
                debug!("Clicked on Azure DevOps Projects header text");
                app.toggle_intents.insert(toggle_key);
            };
        }
    }
}

fn draw_expando_body(app: &mut MyApp, ctx: &Context, ui: &mut Ui) {
    ui.vertical(|ui| match &app.azure_devops_projects {
        Loadable::NotLoaded => {
            ui.label("Not loaded");
        }
        Loadable::Loading => {
            ui.label("Loading...");
        }
        Loadable::Loaded(subs) => {
            let projects = subs.clone();
            for project in projects.iter() {
                draw_entry(app, ctx, ui, project);
            }
        }
        Loadable::Failed(err) => {
            ui.label(&format!("Error: {}", err));
        }
    });
}

fn draw_entry(app: &mut MyApp, ctx: &Context, ui: &mut Ui, project: &AzureDevOpsProject) {
    ui.horizontal(|ui| {
        let label = format!("{}", project.name);
        let checked = app.checkbox_for(&project.id);
        if ui.image(DEVOPS_ICON).clicked() {
            debug!("Clicked on Azure DevOps Project icon");
            *checked ^= true;
        }

        ui.checkbox(checked, label);
    });
}
