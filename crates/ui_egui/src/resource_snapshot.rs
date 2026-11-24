use crate::app::MyApp;
use crate::portal_resource_scope::PortalResourceScope;
use crate::portal_resource_scope::network_security_group_id;
use crate::reflected_value::ReflectedValue;
use crate::storage_account_browser::load_storage_account;
use crate::windows::management_groups_window::load_management_groups;
use crate::windows::resources_window::load_resources;
use crate::workers::load_resource_groups::load_resource_groups;
use crate::workers::load_subscriptions::load_subscriptions;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_app::OutputFormat;
use cloud_terrastodon_azure::AzureNetworkSecurityGroup;
use cloud_terrastodon_azure::AzureNetworkSecurityGroupGetRequest;
use cloud_terrastodon_azure::AzureResourceGroup;
use cloud_terrastodon_azure::AzureStorageAccount;
use cloud_terrastodon_azure::ManagementGroup;
use cloud_terrastodon_azure::Resource;
use cloud_terrastodon_azure::ResourceId;
use cloud_terrastodon_azure::Subscription;
use cloud_terrastodon_azure_cli::storage::account::AzureStorageAccountShowArgs;
use cloud_terrastodon_command::app_work::Loadable;
use cloud_terrastodon_command::app_work::LoadableWorkBuilder;
use cloud_terrastodon_execution::ExecutableRequest;
use eframe::egui::Ui;
use std::sync::Arc;
use std::time::Instant;
use teamy_tracing::gui::DiagnosticReportView;

/// The account's loaded record for a resource, using the richest available schema.
pub(crate) enum ResourceSnapshot {
    ResourceGroup(Arc<AzureResourceGroup>),
    Subscription(Arc<Subscription>),
    ManagementGroup(Arc<ManagementGroup>),
    StorageAccount(Arc<AzureStorageAccount>),
    NetworkSecurityGroup(Arc<AzureNetworkSecurityGroup>),
    Resource(Arc<Resource>),
}

impl ResourceSnapshot {
    pub fn load(app: &mut MyApp, ui: &mut Ui, scope: &PortalResourceScope) -> Option<Self> {
        let mut sign_in = false;
        match scope {
            PortalResourceScope::ResourceGroup(id) => {
                if matches!(app.azure.resource_groups, Loadable::NotLoaded) {
                    load_resource_groups(app, false);
                }
                if let Some(group) = app
                    .azure
                    .resource_groups
                    .as_loaded()
                    .and_then(|groups| groups.get(id))
                {
                    return Some(Self::ResourceGroup(Arc::clone(group)));
                }
                if show_load_status(
                    ui,
                    &app.azure.resource_groups,
                    "Resource group",
                    &mut sign_in,
                ) {
                    load_resource_groups(app, true);
                }
            }
            PortalResourceScope::Subscription(id) => {
                if matches!(app.azure.subscriptions, Loadable::NotLoaded) {
                    load_subscriptions(app);
                }
                if let Some(item) = app
                    .azure
                    .subscriptions
                    .as_loaded()
                    .and_then(|items| items.iter().find(|item| item.id == *id))
                {
                    return Some(Self::Subscription(Arc::clone(item)));
                }
                if show_load_status(ui, &app.azure.subscriptions, "Subscription", &mut sign_in) {
                    load_subscriptions(app);
                }
            }
            PortalResourceScope::ManagementGroup(id) => {
                if matches!(app.azure.management_groups, Loadable::NotLoaded) {
                    load_management_groups(app, false);
                }
                if let Some(item) = app.azure.management_groups.as_loaded().and_then(|items| {
                    items
                        .iter()
                        .find(|item| item.id.name().eq_ignore_ascii_case(id.name()))
                }) {
                    return Some(Self::ManagementGroup(Arc::clone(item)));
                }
                if show_load_status(
                    ui,
                    &app.azure.management_groups,
                    "Management group",
                    &mut sign_in,
                ) {
                    load_management_groups(app, true);
                }
            }
            PortalResourceScope::StorageAccount(id) => {
                if !app.azure.storage_account_browser.entries.contains_key(id) {
                    load_storage_account(app, id, false);
                }
                if let Some(inspection) = app.azure.storage_account_browser.entries.get(id) {
                    if let Some(account) = inspection.response.as_loaded() {
                        return Some(Self::StorageAccount(Arc::clone(account)));
                    }
                    if show_load_status(ui, &inspection.response, "Storage account", &mut sign_in) {
                        load_storage_account(app, id, true);
                    }
                }
            }
            PortalResourceScope::Resource(scope) => {
                if let Some(id) = network_security_group_id(scope) {
                    if !app.azure.network_security_groups.contains_key(id) {
                        load_network_security_group(app, id, false);
                    }
                    if let Some(response) = app.azure.network_security_groups.get(id) {
                        if let Some(group) = response.as_loaded() {
                            return Some(Self::NetworkSecurityGroup(Arc::clone(group)));
                        }
                        if show_load_status(ui, response, "Network security group", &mut sign_in) {
                            load_network_security_group(app, id, true);
                        }
                    }
                } else {
                    if matches!(app.azure.resources, Loadable::NotLoaded) {
                        load_resources(app, false);
                    }
                    if let Some(resource) = app.inventory_resource(scope) {
                        return Some(Self::Resource(Arc::clone(resource)));
                    }
                    if show_load_status(ui, &app.azure.resources, "Resource", &mut sign_in) {
                        load_resources(app, true);
                    }
                }
            }
        }
        if sign_in {
            app.open_selected_account_sign_in();
        }
        None
    }

    pub fn reflected(&self) -> ReflectedValue<'_> {
        match self {
            Self::ResourceGroup(value) => ReflectedValue::new(value.as_ref()),
            Self::Subscription(value) => ReflectedValue::new(value.as_ref()),
            Self::ManagementGroup(value) => ReflectedValue::new(value.as_ref()),
            Self::StorageAccount(value) => ReflectedValue::new(value.as_ref()),
            Self::NetworkSecurityGroup(value) => ReflectedValue::new(value.as_ref()),
            Self::Resource(value) => ReflectedValue::new(value.as_ref()),
        }
    }

    pub fn show_output(&self, app: &mut MyApp, ui: &mut Ui, format: OutputFormat) {
        let output = &mut app.azure.resource_output;
        let cancellation = app.executor.cancellation();
        match self {
            Self::ResourceGroup(value) => {
                output.show(ui, format, value, CliOutput::facet, cancellation)
            }
            Self::Subscription(value) => {
                output.show(ui, format, value, CliOutput::facet, cancellation)
            }
            Self::ManagementGroup(value) => {
                output.show(ui, format, value, CliOutput::facet, cancellation)
            }
            Self::StorageAccount(value) => output.show(
                ui,
                format,
                value,
                AzureStorageAccountShowArgs::output,
                cancellation,
            ),
            Self::NetworkSecurityGroup(value) => {
                output.show(ui, format, value, CliOutput::facet, cancellation)
            }
            Self::Resource(value) => output.show(ui, format, value, CliOutput::facet, cancellation),
        }
    }
}

pub(crate) fn load_network_security_group(app: &mut MyApp, id: &ResourceId, refresh: bool) {
    if matches!(
        app.azure.network_security_groups.get(id),
        Some(Loadable::Loading { .. })
    ) {
        return;
    }
    let Some(authentication) = app.ready_azure_authentication() else {
        return;
    };
    let request = AzureNetworkSecurityGroupGetRequest {
        authentication,
        network_security_group_id: id.clone(),
    };
    let account_generation = app.azure.account_generation;
    let entry_id = id.clone();
    let executor = app.executor.clone();
    let cancellation = executor.cancellation().clone();
    let work_cache = app.work_cache.clone();
    let started_at = Instant::now();
    let enqueue_result = LoadableWorkBuilder::<MyApp, _>::new()
        .description(format!("Load network security group {}", id.resource_name))
        .setter(move |app, response| {
            if let Some(azure) = app.azure_content_mut(account_generation) {
                azure
                    .network_security_groups
                    .insert(entry_id.clone(), response);
            }
        })
        .work(async move |scope| {
            let executor = work_cache.job_executor(&executor.with_cancellation_scope(scope))?;
            request.execute(&executor).with_invalidation(refresh).await
        })
        .build()
        .enqueue(
            &app.work_tracker.clone(),
            app.tx.clone(),
            app,
            &cancellation,
        );
    if let Err(error) = enqueue_result {
        app.azure.network_security_groups.insert(
            id.clone(),
            Loadable::Failed {
                error: error.into(),
                started_at,
                finished_at: Instant::now(),
            },
        );
    }
}

fn show_load_status<T>(ui: &mut Ui, load: &Loadable<T>, label: &str, sign_in: &mut bool) -> bool {
    match load {
        Loadable::NotLoaded | Loadable::Loading { .. } => {
            ui.spinner();
        }
        Loadable::Failed { error, .. } => {
            ui.label(format!("{label} could not be loaded."));
            ui.collapsing("Error details", |ui| {
                DiagnosticReportView::new(error).show(ui);
            });
            return ui
                .horizontal(|ui| {
                    let retry = ui.button("Retry").clicked();
                    *sign_in = ui.button("Sign in again").clicked();
                    retry
                })
                .inner;
        }
        Loadable::Loaded { .. } => {
            ui.label(format!("{label} is unavailable for the selected account."));
        }
    }
    false
}
