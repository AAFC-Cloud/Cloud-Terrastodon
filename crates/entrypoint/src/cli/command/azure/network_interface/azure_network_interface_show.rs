use clap::Args;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_azure::fetch_all_network_interfaces;
use eyre::Result;
use eyre::bail;
use std::io::Write;
use tracing::info;

/// Arguments for showing a single Azure network interface.
#[derive(Args, Debug, Clone)]
pub struct AzureNetworkInterfaceShowArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[arg(long, default_value_t)]
    pub tenant: AzureTenantArgument<'static>,

    /// Network interface resource id, resource name, private IP address, or public IP resource id.
    pub network_interface: String,
}

impl AzureNetworkInterfaceShowArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenant_id = self.tenant.resolve().await?;
        info!(needle = %self.network_interface, %tenant_id, "Fetching Azure network interfaces");
        let network_interfaces = fetch_all_network_interfaces(tenant_id).await?;
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
            0 => bail!("No network interface found matching '{}'.", needle),
            1 => {
                let stdout = std::io::stdout();
                let mut handle = stdout.lock();
                serde_json::to_writer_pretty(&mut handle, &matches.remove(0))?;
                handle.write_all(b"\n")?;
                Ok(())
            }
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
                )
            }
        }
    }
}
