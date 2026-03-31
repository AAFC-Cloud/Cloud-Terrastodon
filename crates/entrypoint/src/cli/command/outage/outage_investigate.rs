use clap::Args;
use cloud_terrastodon_azure::AzurePublicIpResource;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::fetch_all_public_ips;
use eyre::Context;
use eyre::Result;
use eyre::bail;
use hickory_resolver::TokioResolver;
use hickory_resolver::config::ResolverConfig;
use hickory_resolver::config::ResolverOpts;
use hickory_resolver::proto::rr::RData;
use hickory_resolver::proto::rr::RecordType;
use hickory_resolver::system_conf::read_system_conf;
use reqwest::Url;
use serde::Serialize;
use std::collections::BTreeSet;
use std::collections::HashSet;
use std::io::Write;
use std::net::IpAddr;
use tracing::info;

/// Investigate a URL or host by resolving DNS and correlating it with Azure public IPs.
#[derive(Args, Debug, Clone)]
pub struct OutageInvestigateArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[arg(long, default_value_t)]
    pub tenant: AzureTenantArgument<'static>,

    /// URL, host name, or IP address to investigate.
    pub target: String,
}

impl OutageInvestigateArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenant_id = self.tenant.resolve().await?;
        let target_host = extract_target_host(&self.target)?;
        let dns = resolve_target(&target_host).await?;

        info!(%tenant_id, host = %target_host, "Fetching Azure public IP addresses for outage investigation");
        let public_ips = fetch_all_public_ips(tenant_id).await?;
        info!(
            count = public_ips.len(),
            "Fetched Azure public IP addresses"
        );

        let normalized_hosts = std::iter::once(target_host.as_str())
            .chain(std::iter::once(dns.canonical_name.as_str()))
            .chain(dns.aliases.iter().map(String::as_str))
            .map(normalize_host)
            .collect::<HashSet<_>>();
        let resolved_addresses = dns.addresses.iter().copied().collect::<HashSet<_>>();

        let matches = public_ips
            .into_iter()
            .filter_map(|public_ip| {
                match_public_ip(public_ip, &normalized_hosts, &resolved_addresses)
            })
            .collect::<Vec<_>>();

        let report = OutageInvestigationReport {
            input: self.target,
            tenant: tenant_id.to_string(),
            target_host,
            dns,
            matches,
        };

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        serde_json::to_writer_pretty(&mut handle, &report)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}

#[derive(Debug, Serialize)]
struct OutageInvestigationReport {
    input: String,
    tenant: String,
    target_host: String,
    dns: DnsResolution,
    matches: Vec<OutagePublicIpMatch>,
}

#[derive(Debug, Serialize)]
struct DnsResolution {
    canonical_name: String,
    aliases: Vec<String>,
    addresses: Vec<IpAddr>,
}

#[derive(Debug, Serialize)]
struct OutagePublicIpMatch {
    public_ip: AzurePublicIpResource,
    application_gateway_id: Option<String>,
    application_gateway_frontend_ip_configuration_id: Option<String>,
}

fn extract_target_host(target: &str) -> Result<String> {
    let trimmed = target.trim();
    if trimmed.is_empty() {
        bail!("target cannot be empty");
    }

    if let Ok(url) = Url::parse(trimmed) {
        let host = url
            .host_str()
            .ok_or_else(|| eyre::eyre!("URL '{}' does not contain a host", trimmed))?;
        return Ok(trim_fqdn_dot(host));
    }

    if let Some(host) = trimmed.strip_prefix("//") {
        return Ok(trim_fqdn_dot(host));
    }

    Ok(trim_fqdn_dot(trimmed))
}

async fn resolve_target(target_host: &str) -> Result<DnsResolution> {
    if let Ok(ip) = target_host.parse::<IpAddr>() {
        return Ok(DnsResolution {
            canonical_name: target_host.to_string(),
            aliases: Vec::new(),
            addresses: vec![ip],
        });
    }

    let resolver = build_system_resolver().await?;
    let (canonical_name, aliases) = resolve_cname_chain(&resolver, target_host).await?;
    let lookup = resolver
        .lookup_ip(canonical_name.clone())
        .await
        .with_context(|| format!("failed to resolve '{}'", target_host))?;
    let addresses = lookup
        .iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    if addresses.is_empty() {
        bail!("no addresses were found for '{}'", target_host);
    }

    Ok(DnsResolution {
        canonical_name,
        aliases,
        addresses,
    })
}

async fn build_system_resolver() -> Result<TokioResolver> {
    let (config, opts): (ResolverConfig, ResolverOpts) =
        read_system_conf().context("reading system DNS configuration")?;

    Ok(
        TokioResolver::builder_with_config(config, Default::default())
            .with_options(opts)
            .build(),
    )
}

async fn resolve_cname_chain(
    resolver: &TokioResolver,
    query: &str,
) -> Result<(String, Vec<String>)> {
    let mut current = query.to_string();
    let mut aliases = vec![current.clone()];
    let mut seen = HashSet::from([current.clone()]);

    loop {
        let lookup = resolver.lookup(current.clone(), RecordType::CNAME).await;
        let Ok(lookup) = lookup else {
            break;
        };

        let next = lookup.iter().find_map(|record| match record {
            RData::CNAME(name) => Some(trim_fqdn_dot(name.to_utf8())),
            _ => None,
        });
        let Some(next) = next else {
            break;
        };

        if !seen.insert(next.clone()) {
            break;
        }

        current = next.clone();
        aliases.push(next);
    }

    let canonical = current;
    aliases.retain(|alias| alias != &canonical);
    Ok((canonical, aliases))
}

fn match_public_ip(
    public_ip: AzurePublicIpResource,
    normalized_hosts: &HashSet<String>,
    resolved_addresses: &HashSet<IpAddr>,
) -> Option<OutagePublicIpMatch> {
    let dns_match = public_ip
        .properties
        .dns_settings
        .as_ref()
        .and_then(|dns| dns.fqdn.as_deref())
        .map(normalize_host)
        .map(|fqdn| normalized_hosts.contains(&fqdn))
        .unwrap_or(false);
    let ip_match = public_ip
        .properties
        .ip_address
        .map(|ip| resolved_addresses.contains(&ip))
        .unwrap_or(false);

    if !dns_match && !ip_match {
        return None;
    }

    let frontend_ip_configuration_id = public_ip
        .properties
        .ip_configuration
        .as_ref()
        .map(|reference| reference.id.clone());
    let application_gateway_id = frontend_ip_configuration_id
        .as_deref()
        .and_then(application_gateway_id_from_frontend_ip_configuration_id);

    Some(OutagePublicIpMatch {
        public_ip,
        application_gateway_id,
        application_gateway_frontend_ip_configuration_id: frontend_ip_configuration_id,
    })
}

fn application_gateway_id_from_frontend_ip_configuration_id(id: &str) -> Option<String> {
    let marker = "/frontendIPConfigurations/";
    let lower = id.to_ascii_lowercase();
    let marker_index = lower.find(&marker.to_ascii_lowercase())?;
    Some(id[..marker_index].to_string())
}

fn normalize_host(host: impl AsRef<str>) -> String {
    trim_fqdn_dot(host.as_ref()).to_ascii_lowercase()
}

fn trim_fqdn_dot(host: impl AsRef<str>) -> String {
    host.as_ref().trim().trim_end_matches('.').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_host_from_url() -> Result<()> {
        assert_eq!(
            extract_target_host("https://example.com/foo")?,
            "example.com"
        );
        Ok(())
    }

    #[test]
    fn extracts_host_from_bare_host() -> Result<()> {
        assert_eq!(extract_target_host("example.com.")?, "example.com");
        Ok(())
    }

    #[test]
    fn derives_application_gateway_id() {
        let id = "/subscriptions/123/resourceGroups/rg/providers/Microsoft.Network/applicationGateways/agw/frontendIPConfigurations/front";
        assert_eq!(
            application_gateway_id_from_frontend_ip_configuration_id(id).as_deref(),
            Some(
                "/subscriptions/123/resourceGroups/rg/providers/Microsoft.Network/applicationGateways/agw"
            )
        );
    }
}
