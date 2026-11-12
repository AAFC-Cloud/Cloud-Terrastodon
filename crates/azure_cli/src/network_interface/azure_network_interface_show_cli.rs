use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::AzureNetworkInterfaceListRequest;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use eyre::bail;
use facet::Facet;
use tracing::info;

/// Arguments for showing a single Azure network interface.
#[derive(Facet, Debug, Clone)]
pub struct AzureNetworkInterfaceShowArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,

    /// Network interface resource id, resource name, private IP address, or public IP resource id.
    #[facet(figue::positional)]
    pub network_interface: String,
}

impl AzureNetworkInterfaceShowArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let authentication = self.authentication.resolve(executor).await?;
        info!(needle = %self.network_interface, tenant_id = %authentication.tenant_id, "Fetching Azure network interfaces");
        let network_interfaces = AzureNetworkInterfaceListRequest { authentication }
            .execute(executor)
            .await?;
        info!(
            count = network_interfaces.len(),
            "Fetched Azure network interfaces"
        );

        let needle = self.network_interface.trim();
        let mut matches = network_interfaces
            .into_iter()
            .filter(|network_interface| {
                network_interface.id.expanded_form() == needle
                    || network_interface.name.eq_ignore_ascii_case(needle)
                    || network_interface.properties.ip_configurations.iter().any(
                        |ip_configuration| {
                            ip_configuration
                                .properties
                                .private_ip_address
                                .map(|ip| ip.to_string() == needle)
                                .unwrap_or(false)
                                || ip_configuration
                                    .properties
                                    .public_ip_address
                                    .as_ref()
                                    .map(|reference| reference.id.eq_ignore_ascii_case(needle))
                                    .unwrap_or(false)
                        },
                    )
            })
            .collect::<Vec<_>>();

        match matches.len() {
            0 => {
                bail!("No network interface found matching '{}'.", needle);
            }
            1 => Ok(CliOutput::facet(matches.remove(0))),
            _ => {
                matches.sort_by_key(|network_interface| network_interface.id.expanded_form());
                let ids = matches
                    .iter()
                    .map(|network_interface| network_interface.id.expanded_form())
                    .collect::<Vec<_>>()
                    .join("\n  ");
                bail!(
                    "Multiple network interfaces matched '{}'. Use a full resource id.\n  {}",
                    needle,
                    ids
                );
            }
        }
    }
}
