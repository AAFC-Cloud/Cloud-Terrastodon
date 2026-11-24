use super::NOT_REPORTED;
use super::property;
use super::resource_link;
use crate::portal_search::portal_search;
use crate::portal_tab_open::PortalTabOpen;
use cloud_terrastodon_azure::AzureStorageAccountIpRule;
use cloud_terrastodon_azure::AzureStorageAccountNetworkBypassService;
use cloud_terrastodon_azure::AzureStorageAccountNetworkDefaultAction;
use cloud_terrastodon_azure::AzureStorageAccountNetworkRuleSet;
use cloud_terrastodon_azure::AzureStorageAccountProperties;
use cloud_terrastodon_azure::AzureStorageAccountPublicNetworkAccess;
use cloud_terrastodon_azure::AzureStorageAccountResourceAccessRule;
use cloud_terrastodon_azure::AzureStorageAccountVirtualNetworkRule;
use cloud_terrastodon_azure::Scope;
use eframe::egui::Grid;
use eframe::egui::ScrollArea;
use eframe::egui::Ui;
use std::net::IpAddr;
use teamy_tracing::gui::escape_display;

pub(super) fn show_public_access(
    ui: &mut Ui,
    properties: &AzureStorageAccountProperties,
) -> Option<PortalTabOpen> {
    let rules = properties.network_acls.as_ref();
    let default_action = rules.map(|rules| String::from(&rules.default_action));
    Grid::new("storage-public-access")
        .num_columns(2)
        .spacing([22.0, 6.0])
        .show(ui, |ui| {
            property(ui, "Public network access", public_access_label(properties));
            property(
                ui,
                "Firewall default action",
                default_action.as_deref().unwrap_or(NOT_REPORTED),
            );
        });

    let Some(rules) = rules else {
        ui.label("Azure did not report firewall settings.");
        return None;
    };

    ui.add_space(12.0);
    ui.strong("Virtual network rules");
    let virtual_network = show_virtual_network_rules(ui, rules.virtual_network_rules.as_deref());

    ui.add_space(12.0);
    ui.strong("IP rules");
    show_ip_rules(ui, rules);

    ui.add_space(12.0);
    ui.strong("Resource instance rules");
    let resource = show_resource_rules(ui, rules.resource_access_rules.as_deref());

    ui.add_space(12.0);
    ui.strong("Firewall exceptions");
    match &rules.bypass {
        Some(bypass) if bypass.0.is_empty() => {
            ui.label("No exceptions configured.");
        }
        Some(bypass) => {
            for service in &bypass.0 {
                ui.label(escape_display(match service {
                    AzureStorageAccountNetworkBypassService::AzureServices => {
                        "Trusted Microsoft services"
                    }
                    AzureStorageAccountNetworkBypassService::Logging => "Storage logging",
                    AzureStorageAccountNetworkBypassService::Metrics => "Storage metrics",
                    AzureStorageAccountNetworkBypassService::Unknown(value) => value.as_str(),
                }));
            }
        }
        None => {
            ui.weak(NOT_REPORTED);
        }
    }
    resource.or(virtual_network)
}

fn public_access_label(properties: &AzureStorageAccountProperties) -> &str {
    match &properties.public_network_access {
        Some(AzureStorageAccountPublicNetworkAccess::Enabled) => {
            match properties
                .network_acls
                .as_ref()
                .map(|rules| &rules.default_action)
            {
                Some(AzureStorageAccountNetworkDefaultAction::Allow) => "Enabled from all networks",
                Some(AzureStorageAccountNetworkDefaultAction::Deny) => {
                    "Enabled from selected networks"
                }
                _ => "Enabled",
            }
        }
        Some(AzureStorageAccountPublicNetworkAccess::Disabled) => "Disabled",
        Some(AzureStorageAccountPublicNetworkAccess::SecuredByPerimeter) => {
            "Secured by network perimeter"
        }
        Some(AzureStorageAccountPublicNetworkAccess::Unknown(value)) => value.as_str(),
        None => NOT_REPORTED,
    }
}

fn show_virtual_network_rules(
    ui: &mut Ui,
    rules: Option<&[AzureStorageAccountVirtualNetworkRule]>,
) -> Option<PortalTabOpen> {
    let Some(rules) = rules else {
        ui.weak(NOT_REPORTED);
        return None;
    };
    if rules.is_empty() {
        ui.label("No virtual network rules.");
        return None;
    }

    ScrollArea::horizontal()
        .id_salt("storage-virtual-network-scroll")
        .show(ui, |ui| {
            Grid::new("storage-virtual-network-rules")
                .spacing([24.0, 6.0])
                .striped(true)
                .show(ui, |ui| {
                    for heading in [
                        "Virtual network",
                        "Subnet",
                        "Rule state",
                        "Action",
                        "Resource group",
                        "Subscription",
                    ] {
                        ui.strong(heading);
                    }
                    ui.end_row();
                    let mut destination = None;
                    for rule in rules {
                        let network = &rule.id.virtual_network_id;
                        destination = resource_link(ui, &network.as_scope_impl()).or(destination);
                        ui.label(escape_display(rule.id.subnet_name.as_str()))
                            .on_hover_text(escape_display(&rule.id.expanded_form()));
                        let state = rule.state.as_ref().map(String::from);
                        let action = rule.action.as_ref().map(String::from);
                        ui.label(escape_display(state.as_deref().unwrap_or(NOT_REPORTED)));
                        ui.label(escape_display(action.as_deref().unwrap_or(NOT_REPORTED)));
                        destination = resource_link(ui, &network.resource_group_id.as_scope_impl())
                            .or(destination);
                        destination = resource_link(
                            ui,
                            &network.resource_group_id.subscription_id.as_scope_impl(),
                        )
                        .or(destination);
                        ui.end_row();
                    }
                    destination
                })
                .inner
        })
        .inner
}

fn show_resource_rules(
    ui: &mut Ui,
    rules: Option<&[AzureStorageAccountResourceAccessRule]>,
) -> Option<PortalTabOpen> {
    let Some(rules) = rules else {
        ui.weak(NOT_REPORTED);
        return None;
    };
    if rules.is_empty() {
        ui.label("No resource instance rules.");
        return None;
    }

    Grid::new("storage-resource-instance-rules")
        .spacing([24.0, 6.0])
        .striped(true)
        .show(ui, |ui| {
            ui.strong("Resource");
            ui.strong("Tenant");
            ui.end_row();
            let mut destination = None;
            for rule in rules {
                if let Some(resource) = &rule.resource_id {
                    destination = resource_link(ui, resource).or(destination);
                } else {
                    ui.weak(NOT_REPORTED);
                }
                if let Some(tenant) = rule.tenant_id {
                    ui.label(tenant.to_string());
                } else {
                    ui.weak(NOT_REPORTED);
                }
                ui.end_row();
            }
            destination
        })
        .inner
}

fn show_ip_rules(ui: &mut Ui, rules: &AzureStorageAccountNetworkRuleSet) {
    let check_id = ui.make_persistent_id("storage-firewall-ip-check");
    let mut input = ui
        .data(|data| data.get_temp::<String>(check_id))
        .unwrap_or_default();
    if portal_search(ui, &mut input, "Check an IPv4 or IPv6 address", 280.0)
        .on_hover_text("Compare an address with the firewall rules reported by Azure")
        .changed()
    {
        ui.data_mut(|data| data.insert_temp(check_id, input.clone()));
    }
    let mut candidate = None;
    if !input.trim().is_empty() {
        match input.trim().parse::<IpAddr>() {
            Ok(address) => {
                candidate = Some(address);
                let family = if address.is_ipv4() {
                    &rules.ip_rules
                } else {
                    &rules.ipv6_rules
                };
                if let Some(rules) = family {
                    let matching = rules
                        .iter()
                        .filter(|rule| rule.value.contains(address))
                        .count();
                    ui.label(match matching {
                        0 => "No reported IP rules match.".to_owned(),
                        1 => "1 reported IP rule matches.".to_owned(),
                        count => format!("{count} reported IP rules match."),
                    });
                } else {
                    ui.weak("Azure did not report rules for this address family.");
                }
            }
            Err(_) => {
                ui.colored_label(
                    ui.visuals().error_fg_color,
                    "Enter an IPv4 or IPv6 address.",
                );
            }
        }
    }
    show_ip_family(ui, "IPv4", rules.ip_rules.as_deref(), candidate);
    show_ip_family(ui, "IPv6", rules.ipv6_rules.as_deref(), candidate);
}

fn show_ip_family(
    ui: &mut Ui,
    family: &str,
    rules: Option<&[AzureStorageAccountIpRule]>,
    candidate: Option<IpAddr>,
) {
    ui.label(family)
        .on_hover_text("Configured storage firewall address rules");
    let Some(rules) = rules else {
        ui.weak(NOT_REPORTED);
        return;
    };
    if rules.is_empty() {
        ui.label("No rules.");
        return;
    }

    Grid::new(("storage-ip-rules", family))
        .spacing([24.0, 6.0])
        .striped(true)
        .show(ui, |ui| {
            ui.strong("Address or range");
            ui.strong("Action");
            if candidate.is_some() {
                ui.strong("Match");
            }
            ui.end_row();
            for rule in rules {
                ui.label(rule.value.to_string());
                let action = rule.action.as_ref().map(String::from);
                ui.label(escape_display(action.as_deref().unwrap_or(NOT_REPORTED)));
                if let Some(address) = candidate {
                    if rule.value.contains(address) {
                        ui.strong("Matches");
                    } else {
                        ui.weak("—");
                    }
                }
                ui.end_row();
            }
        });
}
