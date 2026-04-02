use crate::AzureLocationName;
use crate::AzureNetworkInterfaceResourceId;
use crate::AzureNetworkInterfaceResourceName;
use crate::AzureTenantId;
use crate::serde_helpers::deserialize_default_if_null;
use crate::serde_helpers::deserialize_none_if_empty_string;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::net::IpAddr;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AzureNetworkInterfaceResource {
    pub id: AzureNetworkInterfaceResourceId,
    pub tenant_id: AzureTenantId,
    pub name: AzureNetworkInterfaceResourceName,
    pub location: AzureLocationName,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_none_if_empty_string")]
    pub managed_by: Option<String>,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub tags: HashMap<String, String>,
    pub properties: AzureNetworkInterfaceResourceProperties,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AzureNetworkInterfaceResourceProperties {
    #[serde(default)]
    pub provisioning_state: Option<String>,
    #[serde(default)]
    pub resource_guid: Option<String>,
    #[serde(default)]
    pub virtual_machine: Option<AzureNetworkInterfaceResourceReference>,
    #[serde(default)]
    pub network_security_group: Option<AzureNetworkInterfaceResourceReference>,
    #[serde(default)]
    pub mac_address: Option<String>,
    #[serde(default)]
    pub primary: Option<bool>,
    #[serde(default)]
    pub enable_accelerated_networking: Option<bool>,
    #[serde(default)]
    pub enable_ip_forwarding: Option<bool>,
    #[serde(default)]
    pub dns_settings: Option<AzureNetworkInterfaceDnsSettings>,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub ip_configurations: Vec<AzureNetworkInterfaceIpConfiguration>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AzureNetworkInterfaceDnsSettings {
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub dns_servers: Vec<String>,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub applied_dns_servers: Vec<String>,
    #[serde(default)]
    pub internal_dns_name_label: Option<String>,
    #[serde(default)]
    pub internal_fqdn: Option<String>,
    #[serde(default)]
    pub internal_domain_name_suffix: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct AzureNetworkInterfaceIpConfiguration {
    pub name: String,
    pub id: String,
    #[serde(default)]
    pub etag: Option<String>,
    #[serde(rename = "type")]
    #[serde(default)]
    pub resource_type: Option<String>,
    pub properties: AzureNetworkInterfaceIpConfigurationProperties,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AzureNetworkInterfaceIpConfigurationProperties {
    #[serde(default)]
    pub provisioning_state: Option<String>,
    #[serde(rename = "privateIPAddress")]
    #[serde(default)]
    pub private_ip_address: Option<IpAddr>,
    #[serde(rename = "privateIPAllocationMethod")]
    #[serde(default)]
    pub private_ip_allocation_method: Option<String>,
    #[serde(rename = "privateIPAddressVersion")]
    #[serde(default)]
    pub private_ip_address_version: Option<String>,
    #[serde(default)]
    pub primary: Option<bool>,
    #[serde(rename = "publicIPAddress")]
    #[serde(default)]
    pub public_ip_address: Option<AzureNetworkInterfaceResourceReference>,
    #[serde(default)]
    pub subnet: Option<AzureNetworkInterfaceResourceReference>,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub application_gateway_backend_address_pools: Vec<AzureNetworkInterfaceResourceReference>,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub load_balancer_backend_address_pools: Vec<AzureNetworkInterfaceResourceReference>,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub load_balancer_inbound_nat_rules: Vec<AzureNetworkInterfaceResourceReference>,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub application_security_groups: Vec<AzureNetworkInterfaceResourceReference>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct AzureNetworkInterfaceResourceReference {
    pub id: String,
}

#[cfg(test)]
mod tests {
    use super::AzureNetworkInterfaceResource;

    #[test]
    fn deserializes_network_interface_resource() -> eyre::Result<()> {
        let resource = serde_json::from_str::<AzureNetworkInterfaceResource>(
            r#"
            {
              "id": "/subscriptions/11111111-1111-1111-1111-111111111111/resourceGroups/my-rg/providers/Microsoft.Network/networkInterfaces/my-nic",
              "tenantId": "22222222-2222-2222-2222-222222222222",
              "name": "my-nic",
              "location": "canadacentral",
              "managedBy": "/subscriptions/11111111-1111-1111-1111-111111111111/resourceGroups/my-rg/providers/Microsoft.Network/privateEndpoints/my-private-endpoint",
              "tags": {
                "env": "test"
              },
              "properties": {
                "provisioningState": "Succeeded",
                "resourceGuid": "33333333-3333-3333-3333-333333333333",
                "macAddress": "00-11-22-33-44-55",
                "enableAcceleratedNetworking": true,
                "ipConfigurations": [
                  {
                    "name": "ipconfig1",
                    "id": "/subscriptions/11111111-1111-1111-1111-111111111111/resourceGroups/my-rg/providers/Microsoft.Network/networkInterfaces/my-nic/ipConfigurations/ipconfig1",
                    "properties": {
                      "privateIPAddress": "10.0.0.5",
                      "privateIPAllocationMethod": "Dynamic",
                      "primary": true,
                      "publicIPAddress": {
                        "id": "/subscriptions/11111111-1111-1111-1111-111111111111/resourceGroups/my-rg/providers/Microsoft.Network/publicIPAddresses/my-pip"
                      },
                      "subnet": {
                        "id": "/subscriptions/11111111-1111-1111-1111-111111111111/resourceGroups/my-rg/providers/Microsoft.Network/virtualNetworks/my-vnet/subnets/default"
                      }
                    }
                  }
                ]
              }
            }
            "#,
        )?;

        assert_eq!(resource.name.to_string(), "my-nic");
        assert_eq!(
            resource.managed_by.as_deref(),
            Some(
                "/subscriptions/11111111-1111-1111-1111-111111111111/resourceGroups/my-rg/providers/Microsoft.Network/privateEndpoints/my-private-endpoint"
            )
        );
        assert_eq!(resource.properties.ip_configurations.len(), 1);
        let ip_configuration = &resource.properties.ip_configurations[0];
        assert_eq!(ip_configuration.name, "ipconfig1");
        assert_eq!(
            ip_configuration
                .properties
                .private_ip_address
                .map(|ip| ip.to_string()),
            Some("10.0.0.5".to_string())
        );
        assert_eq!(
            ip_configuration
                .properties
                .public_ip_address
                .as_ref()
                .map(|reference| reference.id.as_str()),
            Some(
                "/subscriptions/11111111-1111-1111-1111-111111111111/resourceGroups/my-rg/providers/Microsoft.Network/publicIPAddresses/my-pip"
            )
        );

        Ok(())
    }
}
