use super::NOT_REPORTED;
use super::resource_link;
use crate::portal_tab_open::PortalTabOpen;
use cloud_terrastodon_azure::AzureStorageAccountProperties;
use cloud_terrastodon_azure::Scope;
use eframe::egui::Grid;
use eframe::egui::ScrollArea;
use eframe::egui::Ui;
use teamy_tracing::gui::escape_display;

pub(super) fn show_private_endpoints(
    ui: &mut Ui,
    properties: &AzureStorageAccountProperties,
) -> Option<PortalTabOpen> {
    let Some(connections) = &properties.private_endpoint_connections else {
        ui.label("Azure did not report private endpoint connections.");
        return None;
    };
    if connections.is_empty() {
        ui.label("No private endpoint connections.");
        return None;
    }

    ScrollArea::horizontal()
        .id_salt("storage-private-endpoints")
        .show(ui, |ui| {
            Grid::new("storage-private-endpoint-connections")
                .spacing([24.0, 6.0])
                .striped(true)
                .show(ui, |ui| {
                    for heading in [
                        "Private endpoint",
                        "Connection",
                        "Approval",
                        "Provisioning state",
                    ] {
                        ui.strong(heading);
                    }
                    ui.end_row();
                    let mut destination = None;
                    for connection in connections {
                        let properties = connection.properties.as_ref();
                        if let Some(endpoint) = properties
                            .and_then(|properties| properties.private_endpoint.as_ref())
                            .and_then(|endpoint| endpoint.id.as_ref())
                        {
                            destination =
                                resource_link(ui, &endpoint.as_scope_impl()).or(destination);
                        } else {
                            ui.weak(NOT_REPORTED);
                        }
                        ui.label(escape_display(
                            connection.name.as_deref().unwrap_or(NOT_REPORTED),
                        ));
                        let state = properties
                            .map(|properties| &properties.private_link_service_connection_state);
                        let status = state
                            .and_then(|state| state.status.as_ref())
                            .map(String::from);
                        let response =
                            ui.label(escape_display(status.as_deref().unwrap_or(NOT_REPORTED)));
                        if let Some(description) = state
                            .and_then(|state| state.description.as_deref())
                            .filter(|description| !description.is_empty())
                        {
                            response.on_hover_text(escape_display(description));
                        }
                        let provisioning_state = properties
                            .and_then(|properties| properties.provisioning_state.as_ref())
                            .map(String::from);
                        ui.label(escape_display(
                            provisioning_state.as_deref().unwrap_or(NOT_REPORTED),
                        ));
                        ui.end_row();
                    }
                    destination
                })
                .inner
        })
        .inner
}
