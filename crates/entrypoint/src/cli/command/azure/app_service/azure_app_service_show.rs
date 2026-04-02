use clap::Args;
use cloud_terrastodon_azure::AzureAppServiceResource;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_azure::fetch_all_app_services;
use eyre::Result;
use eyre::bail;
use std::io::Write;
use tracing::info;

/// Arguments for showing a single Azure App Service.
#[derive(Args, Debug, Clone)]
pub struct AzureAppServiceShowArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[arg(long, default_value_t)]
    pub tenant: AzureTenantArgument<'static>,

    /// App Service resource id, resource name, hostname, private endpoint id, or inbound IP address.
    pub app_service: String,
}

impl AzureAppServiceShowArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenant_id = self.tenant.resolve().await?;
        info!(needle = %self.app_service, %tenant_id, "Fetching app services");
        let app_services = fetch_all_app_services(tenant_id).await?;
        info!(count = app_services.len(), "Fetched app services");

        let needle = self.app_service.trim();
        let mut matches = app_services
            .into_iter()
            .filter(|app_service| matches_app_service(app_service, needle))
            .collect::<Vec<_>>();

        match matches.len() {
            0 => bail!("No app service found matching '{}'.", needle),
            1 => {
                let stdout = std::io::stdout();
                let mut handle = stdout.lock();
                serde_json::to_writer_pretty(&mut handle, &matches.remove(0))?;
                handle.write_all(b"\n")?;
                Ok(())
            }
            _ => {
                matches.sort_by_key(|app_service| app_service.id.expanded_form());
                let ids = matches
                    .iter()
                    .map(|app_service| app_service.id.expanded_form())
                    .collect::<Vec<_>>()
                    .join("\n  ");
                bail!(
                    "Multiple app services matched '{}'. Use a full resource id.\n  {}",
                    needle,
                    ids
                )
            }
        }
    }
}

fn matches_app_service(app_service: &AzureAppServiceResource, needle: &str) -> bool {
    app_service.id.expanded_form() == needle
        || app_service.name.eq_ignore_ascii_case(needle)
        || app_service
            .properties
            .default_host_name
            .as_deref()
            .map(|host_name| host_name.eq_ignore_ascii_case(needle))
            .unwrap_or(false)
        || app_service
            .properties
            .host_names
            .iter()
            .any(|host_name| host_name.eq_ignore_ascii_case(needle))
        || app_service
            .properties
            .enabled_host_names
            .iter()
            .any(|host_name| host_name.eq_ignore_ascii_case(needle))
        || app_service
            .properties
            .inbound_ip_address
            .map(|ip| ip.to_string() == needle)
            .unwrap_or(false)
        || app_service
            .properties
            .private_endpoint_connections
            .iter()
            .any(|connection| {
                connection.id.eq_ignore_ascii_case(needle)
                    || connection
                        .properties
                        .private_endpoint
                        .as_ref()
                        .map(|reference| reference.id.eq_ignore_ascii_case(needle))
                        .unwrap_or(false)
            })
}

#[cfg(test)]
mod tests {
    use super::matches_app_service;
    use arbitrary::Arbitrary;
    use arbitrary::Unstructured;
    use cloud_terrastodon_azure::AzureAppServicePrivateEndpointConnection;
    use cloud_terrastodon_azure::AzureAppServicePrivateEndpointConnectionProperties;
    use cloud_terrastodon_azure::AzureAppServiceResource;
    use cloud_terrastodon_azure::AzureAppServiceResourceReference;
    use std::net::IpAddr;

    fn sample_app_service() -> AzureAppServiceResource {
        let data = (0u8..=255).cycle().take(4096).collect::<Vec<_>>();
        let mut unstructured = Unstructured::new(&data);
        AzureAppServiceResource::arbitrary(&mut unstructured)
            .expect("sample app service should be generated from arbitrary")
    }

    #[test]
    fn matches_by_hostname() {
        let mut app_service = sample_app_service();
        app_service.properties.host_names = vec![
            "my-app.agr.gc.ca".to_string(),
            "my-app-service.azurewebsites.net".to_string(),
        ];

        assert!(matches_app_service(&app_service, "my-app.agr.gc.ca"));
    }

    #[test]
    fn matches_by_private_endpoint_id() {
        let mut app_service = sample_app_service();
        app_service.properties.private_endpoint_connections = vec![
            AzureAppServicePrivateEndpointConnection {
                name: "my-pe".to_string(),
                resource_type: Some(
                    "Microsoft.Web/sites/privateEndpointConnections".to_string(),
                ),
                id: "/subscriptions/b0e5b743-7da5-4b0c-8d71-b96fea053212/resourceGroups/my-resource-group/providers/Microsoft.Web/sites/my-app-service/privateEndpointConnections/my-pe".to_string(),
                location: None,
                properties: AzureAppServicePrivateEndpointConnectionProperties {
                    provisioning_state: None,
                    private_link_service_connection_state: None,
                    private_endpoint: Some(AzureAppServiceResourceReference {
                        id: "/subscriptions/d1d23c89-f50f-4427-aeb1-d9739d3ebc4e/resourceGroups/my-resource-group/providers/Microsoft.Network/privateEndpoints/my-endpoint".to_string(),
                    }),
                    ip_addresses: vec![
                        "149.105.220.131"
                            .parse::<IpAddr>()
                            .expect("test IP should parse"),
                    ],
                    group_ids: Vec::new(),
                    additional_properties: Default::default(),
                },
            },
        ];

        assert!(matches_app_service(
            &app_service,
            "/subscriptions/d1d23c89-f50f-4427-aeb1-d9739d3ebc4e/resourceGroups/my-resource-group/providers/Microsoft.Network/privateEndpoints/my-endpoint"
        ));
    }
}
