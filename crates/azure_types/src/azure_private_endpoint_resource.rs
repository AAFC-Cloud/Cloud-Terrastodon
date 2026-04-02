use crate::AzureLocationName;
use crate::AzureNetworkInterfaceResourceId;
use crate::AzureNetworkInterfaceResourceName;
use crate::AzurePrivateEndpointResourceId;
use crate::AzurePrivateEndpointResourceName;
use crate::AzureTenantId;
use crate::SubnetId;
use crate::serde_helpers::deserialize_default_if_null;
use crate::serde_helpers::deserialize_none_if_empty_string;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::net::IpAddr;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AzurePrivateEndpointResource {
    pub id: AzurePrivateEndpointResourceId,
    pub tenant_id: AzureTenantId,
    pub name: AzurePrivateEndpointResourceName,
    pub location: AzureLocationName,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub tags: HashMap<String, String>,
    pub properties: AzurePrivateEndpointResourceProperties,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AzurePrivateEndpointResourceProperties {
    #[serde(default)]
    pub provisioning_state: Option<String>,
    #[serde(default)]
    pub resource_guid: Option<String>,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub ip_configurations: Vec<serde_json::Value>,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub network_interfaces: Vec<AzurePrivateEndpointNetworkInterfaceReference>,
    #[serde(default)]
    pub subnet: Option<AzurePrivateEndpointSubnetReference>,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub manual_private_link_service_connections:
        Vec<AzurePrivateEndpointPrivateLinkServiceConnection>,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub private_link_service_connections: Vec<AzurePrivateEndpointPrivateLinkServiceConnection>,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_none_if_empty_string")]
    pub custom_network_interface_name: Option<AzureNetworkInterfaceResourceName>,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub custom_dns_configs: Vec<AzurePrivateEndpointCustomDnsConfig>,
    #[serde(default)]
    pub ip_version_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct AzurePrivateEndpointNetworkInterfaceReference {
    pub id: AzureNetworkInterfaceResourceId,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct AzurePrivateEndpointSubnetReference {
    pub id: SubnetId,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct AzurePrivateEndpointPrivateLinkServiceConnection {
    pub name: String,
    pub id: String,
    #[serde(default)]
    pub etag: Option<String>,
    #[serde(rename = "type")]
    #[serde(default)]
    pub resource_type: Option<String>,
    pub properties: AzurePrivateEndpointPrivateLinkServiceConnectionProperties,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AzurePrivateEndpointPrivateLinkServiceConnectionProperties {
    #[serde(default)]
    pub provisioning_state: Option<String>,
    #[serde(default)]
    pub private_link_service_connection_state:
        Option<AzurePrivateEndpointPrivateLinkServiceConnectionState>,
    #[serde(default)]
    pub private_link_service_id: Option<String>,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub group_ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AzurePrivateEndpointPrivateLinkServiceConnectionState {
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub actions_required: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AzurePrivateEndpointCustomDnsConfig {
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub ip_addresses: Vec<IpAddr>,
    #[serde(default)]
    pub fqdn: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::AzurePrivateEndpointResource;
    use crate::scopes::Scope;

    #[test]
    fn deserializes_private_endpoint_resource() -> eyre::Result<()> {
        let resource = serde_json::from_str::<AzurePrivateEndpointResource>(
            r#"
            {
              "id": "/subscriptions/11111111-1111-1111-1111-111111111111/resourceGroups/my-rg/providers/Microsoft.Network/privateEndpoints/my-private-endpoint",
              "tenantId": "22222222-2222-2222-2222-222222222222",
              "name": "my-private-endpoint",
              "location": "canadacentral",
              "tags": {
                "env": "test"
              },
              "properties": {
                "provisioningState": "Succeeded",
                "resourceGuid": "a11984d9-36b0-472c-8bcc-ad7f6cfe8fba",
                "ipConfigurations": [],
                "networkInterfaces": [
                  {
                    "id": "/subscriptions/11111111-1111-1111-1111-111111111111/resourceGroups/my-rg/providers/Microsoft.Network/networkInterfaces/my-private-endpoint"
                  }
                ],
                "subnet": {
                  "id": "/subscriptions/11111111-1111-1111-1111-111111111111/resourceGroups/core-rg/providers/Microsoft.Network/virtualNetworks/my-vnet/subnets/my-subnet"
                },
                "manualPrivateLinkServiceConnections": [],
                "privateLinkServiceConnections": [
                  {
                    "properties": {
                      "provisioningState": "Succeeded",
                      "privateLinkServiceConnectionState": {
                        "description": "Auto-Approved",
                        "status": "Approved",
                        "actionsRequired": "None"
                      },
                      "privateLinkServiceId": "/subscriptions/11111111-1111-1111-1111-111111111111/resourceGroups/my-rg/providers/Microsoft.Storage/storageAccounts/mystorageaccount",
                      "groupIds": [
                        "blob"
                      ]
                    },
                    "name": "mystorageaccount-privateserviceconnection",
                    "type": "Microsoft.Network/privateEndpoints/privateLinkServiceConnections",
                    "id": "/subscriptions/11111111-1111-1111-1111-111111111111/resourceGroups/my-rg/providers/Microsoft.Network/privateEndpoints/my-private-endpoint/privateLinkServiceConnections/mystorageaccount-privateserviceconnection",
                    "etag": "W/\"33306700-6587-4d4a-9bf1-2e88209237b7\""
                  }
                ],
                "customNetworkInterfaceName": "my-private-endpoint",
                "customDnsConfigs": [
                  {
                    "ipAddresses": [
                      "10.123.123.123"
                    ],
                    "fqdn": "mystorageaccount.blob.core.windows.net"
                  }
                ],
                "ipVersionType": "IPv4"
              }
            }
            "#,
        )?;

        assert_eq!(resource.name.to_string(), "my-private-endpoint");
        assert_eq!(resource.properties.network_interfaces.len(), 1);
        assert_eq!(
            resource.properties.network_interfaces[0].id.expanded_form(),
            "/subscriptions/11111111-1111-1111-1111-111111111111/resourceGroups/my-rg/providers/Microsoft.Network/networkInterfaces/my-private-endpoint"
        );
        assert_eq!(
            resource
                .properties
                .custom_network_interface_name
                .as_ref()
                .map(ToString::to_string),
            Some("my-private-endpoint".to_string())
        );
        assert_eq!(resource.properties.custom_dns_configs.len(), 1);
        assert_eq!(
            resource.properties.custom_dns_configs[0]
                .ip_addresses
                .first()
                .map(ToString::to_string),
            Some("10.123.123.123".to_string())
        );
        assert_eq!(
            resource.properties.private_link_service_connections[0]
                .properties
                .private_link_service_id
                .as_deref(),
            Some(
                "/subscriptions/11111111-1111-1111-1111-111111111111/resourceGroups/my-rg/providers/Microsoft.Storage/storageAccounts/mystorageaccount"
            )
        );

        Ok(())
    }
}
