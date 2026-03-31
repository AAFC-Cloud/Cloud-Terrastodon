use clap::Args;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_azure::fetch_all_public_ips;
use eyre::Result;
use eyre::bail;
use std::io::Write;
use tracing::info;

/// Arguments for showing a single Azure public IP address.
#[derive(Args, Debug, Clone)]
pub struct AzurePublicIpShowArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[arg(long, default_value_t)]
    pub tenant: AzureTenantArgument<'static>,

    /// Public IP resource id, resource name, IP address, or FQDN.
    pub public_ip: String,
}

impl AzurePublicIpShowArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenant_id = self.tenant.resolve().await?;
        info!(needle = %self.public_ip, %tenant_id, "Fetching Azure public IP addresses");
        let public_ips = fetch_all_public_ips(tenant_id).await?;
        info!(
            count = public_ips.len(),
            "Fetched Azure public IP addresses"
        );

        let needle = self.public_ip.trim();
        let mut matches = public_ips
            .into_iter()
            .filter(|public_ip| {
                public_ip.id.expanded_form() == needle
                    || public_ip.name.eq_ignore_ascii_case(needle)
                    || public_ip
                        .properties
                        .ip_address
                        .map(|ip| ip.to_string() == needle)
                        .unwrap_or(false)
                    || public_ip
                        .properties
                        .dns_settings
                        .as_ref()
                        .and_then(|dns| dns.fqdn.as_deref())
                        .map(|fqdn| fqdn.eq_ignore_ascii_case(needle))
                        .unwrap_or(false)
            })
            .collect::<Vec<_>>();

        match matches.len() {
            0 => bail!("No public IP found matching '{}'.", needle),
            1 => {
                let stdout = std::io::stdout();
                let mut handle = stdout.lock();
                serde_json::to_writer_pretty(&mut handle, &matches.remove(0))?;
                handle.write_all(b"\n")?;
                Ok(())
            }
            _ => {
                matches.sort_by_key(|public_ip| public_ip.id.expanded_form());
                let ids = matches
                    .iter()
                    .map(|public_ip| public_ip.id.expanded_form())
                    .collect::<Vec<_>>()
                    .join("\n  ");
                bail!(
                    "Multiple public IPs matched '{}'. Use a full resource id.\n  {}",
                    needle,
                    ids
                )
            }
        }
    }
}
