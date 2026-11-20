use crate::app::MyApp;
use crate::icons::page_icon;
use crate::inventory_table::show_inventory_table;
use crate::portal_page::PortalPage;
use crate::portal_tab_target::PortalTabTarget;
use crate::reflected_value::ReflectedValue;
use crate::table_row_background::TableRowBackground;
use cloud_terrastodon_azure::AzureManagementGroupListRequest;
use cloud_terrastodon_azure::ManagementGroupId;
use cloud_terrastodon_command::app_work::Loadable;
use cloud_terrastodon_command::app_work::LoadableWorkBuilder;
use cloud_terrastodon_execution::ExecutableRequest;
use eframe::egui::Button;
use eframe::egui::Image;
use eframe::egui::Label;
use eframe::egui::ScrollArea;
use eframe::egui::Ui;
use eframe::egui::Vec2;
use egui_extras::Column;
use std::time::Instant;
use teamy_tracing::gui::DiagnosticReportView;

/// Browse the selected tenant's management groups and their ancestors.
pub(crate) fn management_groups_ui(app: &mut MyApp, ui: &mut Ui) {
    if app.show_azure_authentication_required(ui) {
        return;
    }
    if matches!(app.management_groups, Loadable::NotLoaded) {
        load_management_groups(app, false);
    }

    let refresh = ui
        .add_enabled(
            !matches!(app.management_groups, Loadable::Loading { .. }),
            Button::new("Refresh"),
        )
        .clicked();
    let mut next = None;
    match &app.management_groups {
        Loadable::NotLoaded | Loadable::Loading { .. } => {
            ui.spinner();
        }
        Loadable::Failed { error, .. } => {
            ui.label("Could not load management groups");
            ui.collapsing("Error details", |ui| {
                DiagnosticReportView::new(error).show(ui);
            });
            if ui.button("Sign in again").clicked() {
                app.open_selected_account_sign_in();
            }
        }
        Loadable::Loaded { value: groups, .. } => {
            show_inventory_table(ui, |table| {
                table
                    .column(Column::initial(200.0).at_least(125.0).clip(true))
                    .column(Column::initial(190.0).at_least(110.0).clip(true))
                    .column(Column::remainder().at_least(105.0).clip(true))
                    .header(28.0, |mut header| {
                        for heading in ["Name", "Group ID", "Ancestors"] {
                            header.col(|ui| {
                                ui.strong(heading);
                            });
                        }
                    })
                    .body(|mut body| {
                        let background = TableRowBackground::new(&mut body);
                        body.rows(28.0, groups.len(), |mut row| {
                            let group = &groups[row.index()];
                            background.first_cell(&mut row, |ui| {
                                ui.add(
                                    Image::new(page_icon(PortalPage::ManagementGroups))
                                        .fit_to_exact_size(Vec2::splat(18.0)),
                                );
                                if ui
                                    .link(&group.display_name)
                                    .on_hover_text(&group.display_name)
                                    .clicked()
                                {
                                    next = Some(group.id.clone());
                                }
                                false
                            });
                            row.col(|ui| {
                                ui.add(Label::new(group.name()).truncate())
                                    .on_hover_text(group.id.to_string())
                                    .context_menu(|ui| {
                                        if ui.button("Copy resource ID").clicked() {
                                            ui.ctx().copy_text(group.id.to_string());
                                            ui.close();
                                        }
                                    });
                            });
                            row.col(|ui| {
                                let ancestors = group
                                    .management_group_ancestors_chain
                                    .iter()
                                    .map(|ancestor| ancestor.display_name.as_str())
                                    .collect::<Vec<_>>()
                                    .join(", ");
                                ui.add(Label::new(&ancestors).truncate())
                                    .on_hover_text(&ancestors);
                            });
                        });
                    });
            });
        }
    }
    if refresh {
        load_management_groups(app, true);
    }
    if let Some(group) = next {
        app.navigate_tab(PortalTabTarget::ManagementGroup(group));
    }
}

pub(crate) fn management_group_details_ui(app: &mut MyApp, ui: &mut Ui, id: &ManagementGroupId) {
    if app.show_azure_authentication_required(ui) {
        return;
    }
    if matches!(app.management_groups, Loadable::NotLoaded) {
        load_management_groups(app, false);
    }
    if let Some(group) = app.management_group(id) {
        ui.push_id(id, |ui| {
            ScrollArea::vertical().show(ui, |ui| ReflectedValue::new(group).show(ui));
        });
        return;
    }
    ui.horizontal_wrapped(|ui| {
        ui.monospace(id.to_string());
        if ui.small_button("Copy resource ID").clicked() {
            ui.ctx().copy_text(id.to_string());
        }
    });
    match &app.management_groups {
        Loadable::NotLoaded | Loadable::Loading { .. } => {
            ui.spinner();
        }
        Loadable::Loaded { .. } => {
            ui.label("Group details are unavailable for the selected account.");
        }
        Loadable::Failed { error, .. } => {
            ui.collapsing("Error details", |ui| {
                DiagnosticReportView::new(error).show(ui)
            });
        }
    }
}

pub(crate) fn load_management_groups(app: &mut MyApp, refresh: bool) {
    let Some(authentication) = app.ready_azure_authentication() else {
        return;
    };
    let account_generation = app.account_generation;
    let executor = app.executor.clone();
    let cancellation = executor.cancellation().clone();
    let work_cache = app.work_cache.clone();
    let started_at = Instant::now();
    let enqueue_result = LoadableWorkBuilder::<MyApp, _>::new()
        .description("Load management groups")
        .setter(move |app, groups| {
            if app.account_generation == account_generation {
                app.management_groups = groups;
            }
        })
        .work(async move |scope| {
            let executor = work_cache.job_executor(&executor.with_cancellation_scope(scope))?;
            AzureManagementGroupListRequest { authentication }
                .execute(&executor)
                .with_invalidation(refresh)
                .await
        })
        .build()
        .enqueue(
            &app.work_tracker.clone(),
            app.tx.clone(),
            app,
            &cancellation,
        );
    if let Err(error) = enqueue_result {
        app.management_groups = Loadable::Failed {
            error: error.into(),
            started_at,
            finished_at: Instant::now(),
        };
    }
}
