use crate::AzureLocationName;
use crate::AzureNetworkInterfaceResourceId;
use crate::AzureNetworkInterfaceResourceName;
use crate::AzureTenantId;
use std::collections::HashMap;
use std::net::IpAddr;

#[derive(Debug, PartialEq, Clone, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureNetworkInterfaceResource {
    pub id: AzureNetworkInterfaceResourceId,
    pub tenant_id: AzureTenantId,
    pub name: AzureNetworkInterfaceResourceName,
    pub location: AzureLocationName,
    #[facet(default, opaque, proxy = crate::OptionalNonEmptyStringProxy)]
    pub managed_by: Option<String>,
    #[facet(default, opaque, proxy = crate::StringMapDefaultNullProxy)]
    pub tags: HashMap<String, String>,
    pub properties: AzureNetworkInterfaceResourceProperties,
}

#[derive(Debug, PartialEq, Clone, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureNetworkInterfaceResourceProperties {
    #[facet(default)]
    pub provisioning_state: Option<String>,
    #[facet(default)]
    pub resource_guid: Option<String>,
    #[facet(default)]
    pub virtual_machine: Option<AzureNetworkInterfaceResourceReference>,
    #[facet(default)]
    pub network_security_group: Option<AzureNetworkInterfaceResourceReference>,
    #[facet(default)]
    pub mac_address: Option<String>,
    #[facet(default)]
    pub primary: Option<bool>,
    #[facet(default)]
    pub enable_accelerated_networking: Option<bool>,
    #[facet(default)]
    pub enable_ip_forwarding: Option<bool>,
    #[facet(default)]
    pub dns_settings: Option<AzureNetworkInterfaceDnsSettings>,
    #[facet(
        default,
        opaque,
        proxy = crate::VecDefaultNullProxy<AzureNetworkInterfaceIpConfiguration>
    )]
    pub ip_configurations: Vec<AzureNetworkInterfaceIpConfiguration>,
}

#[derive(Debug, PartialEq, Clone, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureNetworkInterfaceDnsSettings {
    #[facet(default, opaque, proxy = crate::VecDefaultNullProxy<String>)]
    pub dns_servers: Vec<String>,
    #[facet(default, opaque, proxy = crate::VecDefaultNullProxy<String>)]
    pub applied_dns_servers: Vec<String>,
    #[facet(default)]
    pub internal_dns_name_label: Option<String>,
    #[facet(default)]
    pub internal_fqdn: Option<String>,
    #[facet(default)]
    pub internal_domain_name_suffix: Option<String>,
}

#[derive(Debug, PartialEq, Clone, facet::Facet)]
pub struct AzureNetworkInterfaceIpConfiguration {
    pub name: String,
    pub id: String,
    #[facet(default)]
    pub etag: Option<String>,
    #[facet(rename = "type", default)]
    pub resource_type: Option<String>,
    pub properties: AzureNetworkInterfaceIpConfigurationProperties,
}

#[derive(Debug, PartialEq, Clone, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureNetworkInterfaceIpConfigurationProperties {
    #[facet(default)]
    pub provisioning_state: Option<String>,
    #[facet(rename = "privateIPAddress", default, opaque, proxy = crate::OptionalIpAddrProxy)]
    pub private_ip_address: Option<IpAddr>,
    #[facet(rename = "privateIPAllocationMethod", default)]
    pub private_ip_allocation_method: Option<String>,
    #[facet(rename = "privateIPAddressVersion", default)]
    pub private_ip_address_version: Option<String>,
    #[facet(default)]
    pub primary: Option<bool>,
    #[facet(rename = "publicIPAddress", default)]
    pub public_ip_address: Option<AzureNetworkInterfaceResourceReference>,
    #[facet(default)]
    pub subnet: Option<AzureNetworkInterfaceResourceReference>,
    #[facet(
        default,
        opaque,
        proxy = crate::VecDefaultNullProxy<AzureNetworkInterfaceResourceReference>
    )]
    pub application_gateway_backend_address_pools: Vec<AzureNetworkInterfaceResourceReference>,
    #[facet(
        default,
        opaque,
        proxy = crate::VecDefaultNullProxy<AzureNetworkInterfaceResourceReference>
    )]
    pub load_balancer_backend_address_pools: Vec<AzureNetworkInterfaceResourceReference>,
    #[facet(
        default,
        opaque,
        proxy = crate::VecDefaultNullProxy<AzureNetworkInterfaceResourceReference>
    )]
    pub load_balancer_inbound_nat_rules: Vec<AzureNetworkInterfaceResourceReference>,
    #[facet(
        default,
        opaque,
        proxy = crate::VecDefaultNullProxy<AzureNetworkInterfaceResourceReference>
    )]
    pub application_security_groups: Vec<AzureNetworkInterfaceResourceReference>,
}

#[derive(Debug, PartialEq, Clone, facet::Facet)]
pub struct AzureNetworkInterfaceResourceReference {
    pub id: String,
}

#[cfg(test)]
mod tests {
    use super::AzureNetworkInterfaceResource;

    #[test]
    fn deserializes_network_interface_resource() -> eyre::Result<()> {
        let resource = facet_json::from_str::<AzureNetworkInterfaceResource>(
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

        let reparsed = facet_json::from_str::<AzureNetworkInterfaceResource>(
            &facet_json::to_string(&resource)?,
        )?;
        assert_eq!(resource, reparsed);

        Ok(())
    }
}
