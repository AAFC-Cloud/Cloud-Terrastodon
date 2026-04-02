use clap::Args;
use cloud_terrastodon_azure::AzurePrivateEndpointResource;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_azure::fetch_all_private_endpoints;
use eyre::Result;
use eyre::bail;
use std::io::Write;
use tracing::info;

/// Arguments for showing a single Azure private endpoint.
#[derive(Args, Debug, Clone)]
pub struct AzurePrivateEndpointShowArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[arg(long, default_value_t)]
    pub tenant: AzureTenantArgument<'static>,

    /// Private endpoint resource id, resource name, NIC id, custom NIC name, target resource id, private IP address, or FQDN.
    pub private_endpoint: String,
}

impl AzurePrivateEndpointShowArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenant_id = self.tenant.resolve().await?;
        info!(needle = %self.private_endpoint, %tenant_id, "Fetching Azure private endpoints");
        let private_endpoints = fetch_all_private_endpoints(tenant_id).await?;
        info!(
            count = private_endpoints.len(),
            "Fetched Azure private endpoints"
        );

        let needle = self.private_endpoint.trim();
        let mut matches = private_endpoints
            .into_iter()
            .filter(|private_endpoint| matches_private_endpoint(private_endpoint, needle))
            .collect::<Vec<_>>();

        match matches.len() {
            0 => bail!("No private endpoint found matching '{}'.", needle),
            1 => {
                let stdout = std::io::stdout();
                let mut handle = stdout.lock();
                serde_json::to_writer_pretty(&mut handle, &matches.remove(0))?;
                handle.write_all(b"\n")?;
                Ok(())
            }
            _ => {
                matches.sort_by_key(|private_endpoint| private_endpoint.id.expanded_form());
                let ids = matches
                    .iter()
                    .map(|private_endpoint| private_endpoint.id.expanded_form())
                    .collect::<Vec<_>>()
                    .join("\n  ");
                bail!(
                    "Multiple private endpoints matched '{}'. Use a full resource id.\n  {}",
                    needle,
                    ids
                )
            }
        }
    }
}

fn matches_private_endpoint(private_endpoint: &AzurePrivateEndpointResource, needle: &str) -> bool {
    private_endpoint.id.expanded_form() == needle
        || private_endpoint.name.eq_ignore_ascii_case(needle)
        || private_endpoint
            .properties
            .custom_network_interface_name
            .as_ref()
            .map(|name| name.eq_ignore_ascii_case(needle))
            .unwrap_or(false)
        || private_endpoint
            .properties
            .network_interfaces
            .iter()
            .any(|network_interface| {
                network_interface
                    .id
                    .expanded_form()
                    .eq_ignore_ascii_case(needle)
            })
        || private_endpoint
            .properties
            .private_link_service_connections
            .iter()
            .chain(
                private_endpoint
                    .properties
                    .manual_private_link_service_connections
                    .iter(),
            )
            .any(|connection| {
                connection
                    .properties
                    .private_link_service_id
                    .as_deref()
                    .map(|id| id.eq_ignore_ascii_case(needle))
                    .unwrap_or(false)
            })
        || private_endpoint
            .properties
            .custom_dns_configs
            .iter()
            .any(|config| {
                config
                    .ip_addresses
                    .iter()
                    .any(|ip| ip.to_string() == needle)
                    || config
                        .fqdn
                        .as_deref()
                        .map(|fqdn| fqdn.eq_ignore_ascii_case(needle))
                        .unwrap_or(false)
            })
}

#[cfg(test)]
mod tests {
    use super::matches_private_endpoint;
    use cloud_terrastodon_azure::AzurePrivateEndpointResource;

    fn sample_private_endpoint() -> AzurePrivateEndpointResource {
        serde_json::from_str(
            r#"
            {
              "id": "/subscriptions/11111111-1111-1111-1111-111111111111/resourceGroups/my-rg/providers/Microsoft.Network/privateEndpoints/my-private-endpoint",
              "tenantId": "22222222-2222-2222-2222-222222222222",
              "name": "my-private-endpoint",
              "location": "canadacentral",
              "tags": {},
              "properties": {
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
                      "privateLinkServiceId": "/subscriptions/11111111-1111-1111-1111-111111111111/resourceGroups/my-rg/providers/Microsoft.Storage/storageAccounts/mystorageaccount",
                      "groupIds": ["blob"]
                    },
                    "name": "mystorageaccount-privateserviceconnection",
                    "type": "Microsoft.Network/privateEndpoints/privateLinkServiceConnections",
                    "id": "/subscriptions/11111111-1111-1111-1111-111111111111/resourceGroups/my-rg/providers/Microsoft.Network/privateEndpoints/my-private-endpoint/privateLinkServiceConnections/mystorageaccount-privateserviceconnection"
                  }
                ],
                "customNetworkInterfaceName": "my-private-endpoint",
                "customDnsConfigs": [
                  {
                    "ipAddresses": ["10.123.123.123"],
                    "fqdn": "mystorageaccount.blob.core.windows.net"
                  }
                ],
                "ipConfigurations": []
              }
            }
            "#,
        )
        .expect("sample private endpoint should deserialize")
    }

    #[test]
    fn matches_by_nic_id() {
        assert!(matches_private_endpoint(
            &sample_private_endpoint(),
            "/subscriptions/11111111-1111-1111-1111-111111111111/resourceGroups/my-rg/providers/Microsoft.Network/networkInterfaces/my-private-endpoint"
        ));
    }

    #[test]
    fn matches_by_private_dns_ip() {
        assert!(matches_private_endpoint(
            &sample_private_endpoint(),
            "10.123.123.123"
        ));
    }
}
