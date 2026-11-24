use crate::azure_devops_project_data::AzureDevOpsProjectData;
use crate::azure_devops_project_data::WORK_ITEM_LIMIT;
use crate::entra_directory_ui::copyable_id;
use crate::reflected_table::ReflectedTable;
use crate::reflected_table_column::ReflectedTableColumn;
use crate::reflected_value::ReflectedValue;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProject;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectId;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectState;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectVisibility;
use cloud_terrastodon_azure_devops::AzureDevOpsReferenceLinkUrl;
use eframe::egui::Context;
use eframe::egui::Grid;
use eframe::egui::Id;
use eframe::egui::OpenUrl;
use eframe::egui::ScrollArea;
use eframe::egui::Ui;
use std::collections::HashMap;

pub(crate) fn show_azure_devops_project_overview(ui: &mut Ui, project: &AzureDevOpsProject) {
    ScrollArea::vertical().show(ui, |ui| {
        ui.strong("Essentials");
        ui.add_space(6.0);
        Grid::new("azure-devops-project-essentials")
            .num_columns(2)
            .spacing([20.0, 6.0])
            .show(ui, |ui| {
                copyable_id(ui, "Project ID", project.id);
                ui.label("Visibility");
                ui.label(match project.visibility {
                    AzureDevOpsProjectVisibility::Private => "Private",
                    AzureDevOpsProjectVisibility::Public => "Public",
                });
                ui.end_row();
                ui.label("State");
                ui.label(match project.state {
                    AzureDevOpsProjectState::WellFormed => "Well formed",
                });
                ui.end_row();
                ui.label("Last updated (UTC)");
                ui.label(
                    project
                        .last_update_time
                        .format("%Y-%m-%d %H:%M:%S")
                        .to_string(),
                );
                ui.end_row();
                if let Some(description) = project
                    .description
                    .as_deref()
                    .filter(|value| !value.is_empty())
                {
                    ui.label("Description");
                    ui.label(description);
                    ui.end_row();
                }
            });
        ui.separator();
        ui.collapsing("Response", |ui| ReflectedValue::new(project).show(ui));
    });
}

pub(crate) fn show_azure_devops_project_data(
    ui: &mut Ui,
    organization: &AzureDevOpsOrganizationUrl,
    project_id: AzureDevOpsProjectId,
    data: &AzureDevOpsProjectData,
    checked_rows: &mut HashMap<Id, bool>,
) {
    match data {
        AzureDevOpsProjectData::Repositories(repositories) => {
            let id = ui.make_persistent_id("devops-repositories");
            let columns = [
                ReflectedTableColumn::new("Name", "name")
                    .with_identifier("id")
                    .with_row_link(),
                ReflectedTableColumn::new("Default branch", "default_branch").groupable(),
                ReflectedTableColumn::new("Size (bytes)", "size").with_width(112.0),
                ReflectedTableColumn::new("Disabled", "is_disabled").with_width(96.0),
            ];
            let context = ui.ctx().clone();
            ReflectedTable::new(id, &columns)
                .with_search_hint("Search repositories")
                .with_result_count()
                .show_explorable_with_activation(
                    ui,
                    repositories.iter(),
                    |row| id.with(row.id.as_ref()),
                    checked_rows,
                    |row| open_browser_link(&context, &row.web_url),
                );
        }
        AzureDevOpsProjectData::WorkItems(work_items) => {
            let id = ui.make_persistent_id("devops-work-items");
            let columns = [
                ReflectedTableColumn::new("Changed (UTC)", "fields.changed_date").with_width(168.0),
                ReflectedTableColumn::new("Title", "fields.title")
                    .with_identifier("id")
                    .with_row_link(),
                ReflectedTableColumn::new("Type", "fields.work_item_type")
                    .with_width(126.0)
                    .groupable(),
                ReflectedTableColumn::new("State", "fields.state")
                    .with_width(126.0)
                    .groupable(),
                ReflectedTableColumn::new("ID", "id").with_width(100.0),
            ];
            ui.weak(format!(
                "Up to {WORK_ITEM_LIMIT} most recently changed work items"
            ));
            let context = ui.ctx().clone();
            ReflectedTable::new(id, &columns)
                .initially_descending()
                .with_search_hint("Search work items")
                .with_result_count()
                .show_explorable_with_activation(
                    ui,
                    work_items.iter().map(AsRef::as_ref),
                    |row| id.with(row.id),
                    checked_rows,
                    |row| {
                        // https://learn.microsoft.com/azure/devops/organizations/projects/rename-project
                        open_browser_link(
                            &context,
                            &format!("{organization}/{project_id}/_workitems/edit/{}", row.id),
                        );
                    },
                );
        }
        AzureDevOpsProjectData::Groups(groups) => {
            let id = ui.make_persistent_id("devops-groups");
            let columns = [
                ReflectedTableColumn::new("Name", "display_name").with_identifier("principal_name"),
                ReflectedTableColumn::new("Description", "description"),
                ReflectedTableColumn::new("Origin", "origin")
                    .with_width(108.0)
                    .groupable(),
            ];
            ReflectedTable::new(id, &columns)
                .with_search_hint("Search groups")
                .with_result_count()
                .show_explorable(
                    ui,
                    groups.iter().map(AsRef::as_ref),
                    |row| id.with(&row.descriptor),
                    checked_rows,
                );
        }
        AzureDevOpsProjectData::Teams(teams) => {
            let id = ui.make_persistent_id("devops-teams");
            let columns = [
                ReflectedTableColumn::new("Name", "name").with_identifier("id"),
                ReflectedTableColumn::new("Description", "description"),
            ];
            ReflectedTable::new(id, &columns)
                .with_search_hint("Search teams")
                .with_result_count()
                .show_explorable(
                    ui,
                    teams.iter().map(AsRef::as_ref),
                    |row| id.with(row.id.as_ref()),
                    checked_rows,
                );
        }
        AzureDevOpsProjectData::ServiceConnections(connections) => {
            let id = ui.make_persistent_id("devops-service-connections");
            let columns = [
                ReflectedTableColumn::new("Name", "name").with_identifier("id"),
                ReflectedTableColumn::new("Type", "kind")
                    .with_width(124.0)
                    .groupable(),
                ReflectedTableColumn::new("Ready", "is_ready").with_width(96.0),
                ReflectedTableColumn::new("Shared", "is_shared").with_width(96.0),
                ReflectedTableColumn::new("Description", "description"),
            ];
            ReflectedTable::new(id, &columns)
                .with_search_hint("Search service connections")
                .with_result_count()
                .show_explorable(ui, connections.iter(), |row| id.with(row.id), checked_rows);
        }
        AzureDevOpsProjectData::AgentPools(pools) => {
            let id = ui.make_persistent_id("devops-agent-pools");
            let columns = [
                ReflectedTableColumn::new("Name", "name").with_identifier("pool.id"),
                ReflectedTableColumn::new("Type", "pool.pool_type")
                    .with_width(112.0)
                    .groupable(),
                ReflectedTableColumn::new("Hosted", "pool.is_hosted").with_width(96.0),
                ReflectedTableColumn::new("Agent count", "pool.size").with_width(108.0),
                ReflectedTableColumn::new("Queue ID", "id").with_width(96.0),
            ];
            ReflectedTable::new(id, &columns)
                .with_search_hint("Search agent pools")
                .with_result_count()
                .show_explorable(
                    ui,
                    pools.iter().map(AsRef::as_ref),
                    |row| id.with(row.pool.id),
                    checked_rows,
                );
        }
        AzureDevOpsProjectData::Pipelines(pipelines) => {
            let id = ui.make_persistent_id("devops-pipelines");
            let columns = [
                ReflectedTableColumn::new("Name", "name")
                    .with_identifier("id")
                    .with_row_link(),
                ReflectedTableColumn::new("Folder", "path").groupable(),
                ReflectedTableColumn::new("Queue status", "queue_status")
                    .with_width(132.0)
                    .groupable(),
                ReflectedTableColumn::new("Created (UTC)", "created_date").with_width(168.0),
            ];
            let context = ui.ctx().clone();
            ReflectedTable::new(id, &columns)
                .with_search_hint("Search pipelines")
                .with_result_count()
                .show_explorable_with_activation(
                    ui,
                    pipelines.iter().map(AsRef::as_ref),
                    |row| id.with(row.id),
                    checked_rows,
                    |row| {
                        // https://learn.microsoft.com/azure/devops/pipelines/tasks/reference/download-pipeline-artifact-v2
                        open_browser_link(
                            &context,
                            &format!(
                                "{organization}/{project_id}/_build?definitionId={}&_a=summary",
                                row.id
                            ),
                        );
                    },
                );
        }
        AzureDevOpsProjectData::TestPlans(plans) => {
            let id = ui.make_persistent_id("devops-test-plans");
            let columns = [
                ReflectedTableColumn::new("Name", "name").with_identifier("id"),
                ReflectedTableColumn::new("State", "state")
                    .with_width(106.0)
                    .groupable(),
                ReflectedTableColumn::new("Iteration", "iteration").groupable(),
                ReflectedTableColumn::new("Updated (UTC)", "updated_date").with_width(168.0),
            ];
            ReflectedTable::new(id, &columns)
                .with_search_hint("Search test plans")
                .with_result_count()
                .show_explorable(
                    ui,
                    plans.iter().map(AsRef::as_ref),
                    |row| id.with(row.id),
                    checked_rows,
                );
        }
    }
}

fn open_browser_link(context: &Context, link: &str) {
    let Ok(link) = AzureDevOpsReferenceLinkUrl::try_new(link) else {
        return;
    };
    let url = link.as_url();
    if matches!(url.scheme(), "https" | "http")
        && url.username().is_empty()
        && url.password().is_none()
    {
        context.open_url(OpenUrl::new_tab(link.as_str()));
    }
}
