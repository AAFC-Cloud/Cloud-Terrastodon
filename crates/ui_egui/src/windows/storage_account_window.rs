use crate::app::MyApp;
use crate::portal_page::PortalPage;
use crate::portal_resource_output_target::PortalResourceOutputTarget;
use crate::portal_tab_target::PortalTabTarget;
use crate::reflected_value::ReflectedValue;
use crate::resource_detail_header::resource_detail_header_with_output;
use crate::storage_account_browser::load_storage_account;
use cloud_terrastodon_app::OutputFormat;
use cloud_terrastodon_azure::AzureStorageAccount;
use cloud_terrastodon_azure::AzureStorageAccountId;
use cloud_terrastodon_command::app_work::Loadable;
use eframe::egui::CollapsingHeader;
use eframe::egui::Grid;
use eframe::egui::ScrollArea;
use eframe::egui::Ui;
use eframe::egui::Vec2;
use teamy_tracing::gui::DiagnosticReportView;

/// Inspect the same typed request and response as the storage account show command.
pub(crate) fn storage_account_ui(app: &mut MyApp, ui: &mut Ui, id: &AzureStorageAccountId) {
    let (inventory, view_output) = resource_detail_header_with_output(
        ui,
        id.storage_account_name.as_str(),
        "Storage account",
        PortalPage::StorageAccounts,
    );
    if inventory {
        app.navigate(PortalPage::StorageAccounts);
        return;
    }
    if view_output {
        app.navigate_tab(PortalTabTarget::ResourceOutput {
            resource: PortalResourceOutputTarget::StorageAccount(id.clone()),
            format: OutputFormat::Json,
        });
        return;
    }
    if app.show_azure_authentication_required(ui) {
        return;
    }
    if !app.storage_account_browser.entries.contains_key(id) {
        load_storage_account(app, id, false);
    }
    let Some(inspection) = app.storage_account_browser.entries.get(id) else {
        return;
    };
    let mut next_tab = None;
    let mut sign_in = false;
    ui.separator();
    ui.push_id(id, |ui| {
        ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                match &inspection.response {
                    Loadable::NotLoaded | Loadable::Loading { .. } => {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label("Loading storage account…");
                        });
                    }
                    Loadable::Failed { error, .. } => {
                        ui.label("Could not load storage account.");
                        ui.collapsing("Error details", |ui| {
                            DiagnosticReportView::new(error).show(ui);
                        });
                        sign_in = ui.button("Sign in again").clicked();
                    }
                    Loadable::Loaded { value, .. } => {
                        ui.strong("Essentials");
                        next_tab = storage_account_essentials(ui, app, value);
                        ui.separator();
                    }
                }
                ui.collapsing("Request", |ui| {
                    ReflectedValue::new(&inspection.request).show(ui);
                });
                if let Loadable::Loaded { value, .. } = &inspection.response {
                    CollapsingHeader::new("Response")
                        .default_open(true)
                        .show(ui, |ui| {
                            if let Some(target) = ReflectedValue::new(value.as_ref())
                                .show_links(ui, &PortalTabTarget::StorageAccount(id.clone()))
                            {
                                next_tab = Some(target);
                            }
                        });
                }
            });
    });
    if sign_in {
        app.open_selected_account_sign_in();
    }
    if let Some(target) = next_tab {
        app.navigate_tab(target);
    }
}

fn storage_account_essentials(
    ui: &mut Ui,
    app: &MyApp,
    account: &AzureStorageAccount,
) -> Option<PortalTabTarget> {
    let mut next_tab = None;
    let resource_group = &account.id.resource_group_id;
    let subscription = PortalTabTarget::Subscription(resource_group.subscription_id);
    Grid::new("essentials")
        .num_columns(2)
        .spacing(Vec2::new(24.0, 5.0))
        .show(ui, |ui| {
            ui.label("Resource group");
            if ui
                .link(resource_group.resource_group_name.as_str())
                .clicked()
            {
                next_tab = Some(PortalTabTarget::ResourceGroup(resource_group.clone()));
            }
            ui.end_row();

            ui.label("Subscription");
            if ui
                .link(app.portal_tab_title(&subscription).as_ref())
                .clicked()
            {
                next_tab = Some(subscription.clone());
            }
            ui.end_row();

            ui.label("Location");
            ui.label(&account.location);
            ui.end_row();

            ui.label("SKU");
            ui.label(&account.sku.name);
            ui.end_row();

            ui.label("Account kind");
            ui.label(format!("{:?}", account.kind));
            ui.end_row();

            if let Some(state) = &account.properties.provisioning_state {
                ui.label("Provisioning state");
                ui.label(String::from(state));
                ui.end_row();
            }
            if let Some(allowed) = account.properties.allow_blob_public_access {
                ui.label("Allows blob anonymous access");
                ui.label(if allowed { "Yes" } else { "No" });
                ui.end_row();
            }
        });
    next_tab
}
