use crate::AzureAppServiceResourceId;
use crate::AzureAppServiceResourceName;
use crate::AzureLocationName;
use crate::AzureTenantId;
use arbitrary::Arbitrary;
use facet_json::RawJson;
use std::collections::HashMap;
use std::convert::Infallible;
use std::net::IpAddr;

#[derive(Debug, PartialEq, Eq, Clone, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureAppServiceResource {
    pub id: AzureAppServiceResourceId,
    pub tenant_id: AzureTenantId,
    pub name: AzureAppServiceResourceName,
    #[facet(default, proxy = AzureAppServiceResourceKindListProxy)]
    pub kind: Vec<AzureAppServiceResourceKind>,
    pub location: AzureLocationName,
    #[facet(default, proxy = crate::StringMapDefaultNullProxy)]
    pub tags: HashMap<String, String>,
    pub properties: AzureAppServiceResourceProperties,
}

#[derive(Debug, PartialEq, Eq, Clone, Arbitrary, facet::Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureAppServiceResourceKind {
    WorkflowApp,
    AzureContainerApps,
    Container,
    App,
    FunctionApp,
    Linux,
    Other(String),
}

impl TryFrom<String> for AzureAppServiceResourceKind {
    type Error = Infallible;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(match value.as_str() {
            "workflowapp" => Self::WorkflowApp,
            "azurecontainerapps" => Self::AzureContainerApps,
            "container" => Self::Container,
            "app" => Self::App,
            "functionapp" => Self::FunctionApp,
            "linux" => Self::Linux,
            other => Self::Other(other.to_string()),
        })
    }
}

impl From<&AzureAppServiceResourceKind> for String {
    fn from(value: &AzureAppServiceResourceKind) -> Self {
        match value {
            AzureAppServiceResourceKind::WorkflowApp => "workflowapp",
            AzureAppServiceResourceKind::AzureContainerApps => "azurecontainerapps",
            AzureAppServiceResourceKind::Container => "container",
            AzureAppServiceResourceKind::App => "app",
            AzureAppServiceResourceKind::FunctionApp => "functionapp",
            AzureAppServiceResourceKind::Linux => "linux",
            AzureAppServiceResourceKind::Other(value) => return value.clone(),
        }
        .to_string()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, facet::Facet)]
#[facet(transparent)]
pub struct AzureAppServiceResourceKindListProxy(Option<String>);

impl From<AzureAppServiceResourceKindListProxy> for Vec<AzureAppServiceResourceKind> {
    fn from(value: AzureAppServiceResourceKindListProxy) -> Self {
        let Some(raw) = value.0 else {
            return Vec::new();
        };
        raw.split(',')
            .map(str::trim)
            .filter(|kind| !kind.is_empty())
            .map(
                |kind| match AzureAppServiceResourceKind::try_from(kind.to_string()) {
                    Ok(kind) => kind,
                    Err(err) => match err {},
                },
            )
            .collect()
    }
}

impl From<&Vec<AzureAppServiceResourceKind>> for AzureAppServiceResourceKindListProxy {
    fn from(value: &Vec<AzureAppServiceResourceKind>) -> Self {
        if value.is_empty() {
            Self(None)
        } else {
            Self(Some(
                value.iter().map(String::from).collect::<Vec<_>>().join(","),
            ))
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureAppServiceResourceProperties {
    #[facet(default)]
    pub state: Option<String>,
    #[facet(default)]
    pub enabled: Option<bool>,
    #[facet(default)]
    pub public_network_access: Option<String>,
    #[facet(default)]
    pub default_host_name: Option<String>,
    #[facet(default, proxy = crate::VecDefaultNullProxy<String>)]
    pub host_names: Vec<String>,
    #[facet(default, proxy = crate::VecDefaultNullProxy<String>)]
    pub enabled_host_names: Vec<String>,
    #[facet(default, opaque, proxy = crate::OptionalIpAddrProxy)]
    pub inbound_ip_address: Option<IpAddr>,
    #[facet(default, opaque, proxy = crate::OptionalIpAddrProxy)]
    pub inbound_ipv6_address: Option<IpAddr>,
    #[facet(default)]
    pub outbound_ip_addresses: Option<String>,
    #[facet(default)]
    pub possible_outbound_ip_addresses: Option<String>,
    #[facet(default)]
    pub possible_outbound_ipv6_addresses: Option<String>,
    #[facet(default)]
    pub possible_inbound_ip_addresses: Option<String>,
    #[facet(default)]
    pub possible_inbound_ipv6_addresses: Option<String>,
    #[facet(default)]
    pub private_link_identifiers: Option<String>,
    #[facet(default)]
    pub server_farm_id: Option<String>,
    #[facet(default)]
    pub virtual_network_subnet_id: Option<String>,
    #[facet(
        default,
                proxy = crate::VecDefaultNullProxy<AzureAppServicePrivateEndpointConnection>
    )]
    pub private_endpoint_connections: Vec<AzureAppServicePrivateEndpointConnection>,
    #[facet(default)]
    pub site_config: Option<AzureAppServiceSiteConfig>,
    #[arbitrary(default)]
    #[facet(flatten)]
    pub additional_properties: HashMap<String, RawJson<'static>>,
}

#[derive(Debug, PartialEq, Eq, Clone, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureAppServicePrivateEndpointConnection {
    pub name: String,
    #[facet(rename = "type", default)]
    pub resource_type: Option<String>,
    pub id: String,
    #[facet(default)]
    pub location: Option<AzureLocationName>,
    pub properties: AzureAppServicePrivateEndpointConnectionProperties,
}

#[derive(Debug, PartialEq, Eq, Clone, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureAppServicePrivateEndpointConnectionProperties {
    #[facet(default)]
    pub provisioning_state: Option<String>,
    #[facet(default)]
    pub private_link_service_connection_state:
        Option<AzureAppServicePrivateLinkServiceConnectionState>,
    #[facet(default)]
    pub private_endpoint: Option<AzureAppServiceResourceReference>,
    #[facet(default, opaque, proxy = crate::IpAddrVecDefaultNullProxy)]
    pub ip_addresses: Vec<IpAddr>,
    #[facet(default, proxy = crate::VecDefaultNullProxy<String>)]
    pub group_ids: Vec<String>,
    #[arbitrary(default)]
    #[facet(flatten)]
    pub additional_properties: HashMap<String, RawJson<'static>>,
}

#[derive(Debug, PartialEq, Eq, Clone, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureAppServicePrivateLinkServiceConnectionState {
    #[facet(default)]
    pub description: Option<String>,
    #[facet(default)]
    pub status: Option<String>,
    #[facet(default)]
    pub actions_required: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Clone, Arbitrary, facet::Facet)]
pub struct AzureAppServiceResourceReference {
    pub id: String,
}

#[derive(Debug, PartialEq, Eq, Clone, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureAppServiceSiteConfig {
    #[facet(default)]
    pub public_network_access: Option<String>,
    #[facet(default)]
    pub linux_fx_version: Option<String>,
    #[facet(default)]
    pub windows_fx_version: Option<String>,
    #[facet(default)]
    pub always_on: Option<bool>,
    #[arbitrary(default)]
    #[facet(flatten)]
    pub additional_properties: HashMap<String, RawJson<'static>>,
}

#[cfg(test)]
mod tests {
    use super::AzureAppServiceResource;
    use super::AzureAppServiceResourceKind;

    #[test]
    fn deserializes_unknown_app_service_kind_to_other() -> eyre::Result<()> {
        let kind = facet_json::from_str::<AzureAppServiceResourceKind>("\"brandnewkind\"")?;

        assert_eq!(
            kind,
            AzureAppServiceResourceKind::Other("brandnewkind".to_string())
        );

        Ok(())
    }

    #[test]
    fn resource_json_round_trips() -> eyre::Result<()> {
        let json = r#"
        {
            "id": "/subscriptions/00000000-0000-0000-0000-000000000000/resourceGroups/my-rg/providers/Microsoft.Web/sites/my-app-service",
            "tenantId": "11111111-1111-1111-1111-111111111111",
            "name": "my-app-service",
            "kind": "app,linux,brandnewkind",
            "location": "canadacentral",
            "tags": null,
            "properties": {
                "hostNames": null,
                "enabledHostNames": null,
                "privateEndpointConnections": null,
                "customProperty": { "kept": true }
            }
        }
        "#;

        let resource = facet_json::from_str::<AzureAppServiceResource>(json)?;
        assert_eq!(
            resource.kind,
            vec![
                AzureAppServiceResourceKind::App,
                AzureAppServiceResourceKind::Linux,
                AzureAppServiceResourceKind::Other("brandnewkind".to_string())
            ]
        );

        let reparsed =
            facet_json::from_str::<AzureAppServiceResource>(&facet_json::to_string(&resource)?)?;
        assert_eq!(resource, reparsed);

        Ok(())
    }
}

cloud_terrastodon_registry::register_thing!(AzureAppServiceResource);
cloud_terrastodon_registry::register_arbitrary!(AzureAppServiceResource);
cloud_terrastodon_registry::register_arbitrary!(Vec<AzureAppServiceResource>);
