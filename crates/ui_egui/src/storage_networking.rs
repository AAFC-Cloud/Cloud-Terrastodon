use crate::app::MyApp;
use crate::portal_resource_scope::PortalResourceScope;
use crate::portal_section_tabs::portal_section_tabs;
use crate::portal_tab_open::PortalTabOpen;
use crate::portal_tab_target::PortalTabTarget;
use crate::resource_snapshot::ResourceSnapshot;
use cloud_terrastodon_azure::AzureStorageAccountId;
use cloud_terrastodon_azure::AzureStorageAccountProperties;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_azure::ScopeImpl;
use eframe::egui::Grid;
use eframe::egui::ScrollArea;
use eframe::egui::Ui;
use teamy_tracing::gui::escape_display;

mod private_endpoints;
mod public_access;
mod routing;
mod storage_networking_section;

use self::private_endpoints::show_private_endpoints;
use self::public_access::show_public_access;
use self::routing::show_routing;
use self::storage_networking_section::StorageNetworkingSection;

const NOT_REPORTED: &str = "Not reported";

pub(crate) fn storage_networking_ui(app: &mut MyApp, ui: &mut Ui, id: &AzureStorageAccountId) {
    ui.push_id(id, |ui| {
        let section_id = ui.make_persistent_id("storage-networking-section");
        let mut section = ui
            .data(|data| data.get_temp::<StorageNetworkingSection>(section_id))
            .unwrap_or_default();
        let previous = section;
        ui.horizontal_wrapped(|ui| {
            portal_section_tabs(
                ui,
                &mut section,
                StorageNetworkingSection::ALL,
                StorageNetworkingSection::label,
            );
        });
        if section != previous {
            ui.data_mut(|data| data.insert_temp(section_id, section));
        }
        ui.separator();

        let scope = PortalResourceScope::StorageAccount(id.clone());
        let Some(ResourceSnapshot::StorageAccount(account)) =
            ResourceSnapshot::load(app, ui, &scope)
        else {
            return;
        };
        let destination = ScrollArea::vertical()
            .id_salt(section)
            .show(ui, |ui| match section {
                StorageNetworkingSection::PublicAccess => {
                    show_public_access(ui, &account.properties)
                }
                StorageNetworkingSection::PrivateEndpoints => {
                    show_private_endpoints(ui, &account.properties)
                }
                StorageNetworkingSection::Routing => {
                    show_routing(ui, &account.properties);
                    None
                }
                StorageNetworkingSection::CustomDomain => {
                    show_custom_domain(ui, &account.properties);
                    None
                }
            })
            .inner;
        if let Some(destination) = destination {
            destination.apply(app);
        }
    });
}

fn show_custom_domain(ui: &mut Ui, properties: &AzureStorageAccountProperties) {
    let Some(domain) = &properties.custom_domain else {
        ui.label("Azure did not report a custom domain.");
        return;
    };
    Grid::new("storage-custom-domain")
        .num_columns(2)
        .spacing([22.0, 6.0])
        .show(ui, |ui| {
            property(ui, "Domain", &domain.name);
            property(
                ui,
                "Indirect domain validation",
                setting_label(domain.use_sub_domain_name),
            );
        });
}

fn property(ui: &mut Ui, label: &str, value: &str) {
    ui.label(label);
    ui.label(escape_display(value));
    ui.end_row();
}

fn resource_link(ui: &mut Ui, scope: &ScopeImpl) -> Option<PortalTabOpen> {
    let id = scope.expanded_form();
    if !id.starts_with('/') || id == "/" {
        ui.label(escape_display(&id));
        return None;
    }
    let target = PortalTabTarget::from_arm_scope(scope);
    let link = ui
        .link(escape_display(&target.title()))
        .on_hover_text(escape_display(&id));
    PortalTabOpen::from_response(&link, || target)
}

fn setting_label(setting: Option<bool>) -> &'static str {
    match setting {
        Some(true) => "Enabled",
        Some(false) => "Disabled",
        None => NOT_REPORTED,
    }
}
