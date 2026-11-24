use super::NOT_REPORTED;
use super::property;
use super::setting_label;
use crate::reflected_value::ReflectedValue;
use cloud_terrastodon_azure::AzureStorageAccountProperties;
use cloud_terrastodon_azure::AzureStorageAccountRoutingChoice;
use eframe::egui::Grid;
use eframe::egui::Ui;

pub(super) fn show_routing(ui: &mut Ui, properties: &AzureStorageAccountProperties) {
    let routing = properties.routing_preference.as_ref();
    let routing_choice = match routing.and_then(|routing| routing.routing_choice.as_ref()) {
        Some(AzureStorageAccountRoutingChoice::MicrosoftRouting) => "Microsoft network",
        Some(AzureStorageAccountRoutingChoice::InternetRouting) => "Internet",
        Some(AzureStorageAccountRoutingChoice::Unknown(value)) => value.as_str(),
        None => NOT_REPORTED,
    };
    Grid::new("storage-network-routing")
        .num_columns(2)
        .spacing([22.0, 6.0])
        .show(ui, |ui| {
            property(ui, "Routing preference", routing_choice);
            property(
                ui,
                "Publish Microsoft network endpoints",
                setting_label(routing.and_then(|routing| routing.publish_microsoft_endpoints)),
            );
            property(
                ui,
                "Publish internet endpoints",
                setting_label(routing.and_then(|routing| routing.publish_internet_endpoints)),
            );
            property(
                ui,
                "Publish IPv6 endpoints",
                setting_label(
                    properties
                        .dual_stack_endpoint_preference
                        .as_ref()
                        .and_then(|preference| preference.publish_ipv6_endpoint),
                ),
            );
        });

    ui.add_space(12.0);
    ui.strong("Service endpoints");
    for (label, endpoints) in [
        ("Primary", properties.primary_endpoints.as_ref()),
        ("Secondary", properties.secondary_endpoints.as_ref()),
    ] {
        if let Some(endpoints) = endpoints {
            ui.collapsing(label, |ui| ReflectedValue::new(endpoints).show(ui));
        } else {
            ui.horizontal(|ui| {
                ui.label(label);
                ui.weak(NOT_REPORTED);
            });
        }
    }
}
