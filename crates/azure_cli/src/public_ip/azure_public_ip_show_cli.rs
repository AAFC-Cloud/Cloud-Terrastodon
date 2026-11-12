use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::AzurePublicIpListRequest;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use eyre::bail;
use facet::Facet;
use tracing::info;

/// Arguments for showing a single Azure public IP address.
#[derive(Facet, Debug, Clone)]
pub struct AzurePublicIpShowArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,

    /// Public IP resource id, resource name, IP address, or FQDN.
    #[facet(figue::positional)]
    pub public_ip: String,
}

impl AzurePublicIpShowArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let authentication = self.authentication.resolve(executor).await?;
        info!(needle = %self.public_ip, tenant_id = %authentication.tenant_id, "Fetching Azure public IP addresses");
        let public_ips = AzurePublicIpListRequest { authentication }
            .execute(executor)
            .await?;
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
            0 => {
                bail!("No public IP found matching '{}'.", needle);
            }
            1 => Ok(CliOutput::facet(matches.remove(0))),
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
                );
            }
        }
    }
}
