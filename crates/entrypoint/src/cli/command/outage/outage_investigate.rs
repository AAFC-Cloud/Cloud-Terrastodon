use clap::Args;
use cloud_terrastodon_azure::AzureApplicationGatewayResourceBackendHealthResponse;
use cloud_terrastodon_azure::AzureApplicationGatewayResourceBackendHealthServer;
use cloud_terrastodon_azure::AzureApplicationGatewayResourceBackendHealthServerHealth;
use cloud_terrastodon_azure::AzureApplicationGatewayResourceId;
use cloud_terrastodon_azure::AzureNetworkInterfaceResource;
use cloud_terrastodon_azure::AzurePrivateEndpointResource;
use cloud_terrastodon_azure::AzurePrivateEndpointResourceId;
use cloud_terrastodon_azure::AzurePublicIpResource;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_azure::fetch_all_network_interfaces;
use cloud_terrastodon_azure::fetch_all_private_endpoints;
use cloud_terrastodon_azure::fetch_all_public_ips;
use cloud_terrastodon_azure::fetch_application_gateway_backend_health;
use color_eyre::owo_colors::OwoColorize;
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
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashSet;
use std::net::IpAddr;
use std::path::Path;
use std::path::PathBuf;
use tracing::info;

/// Investigate a URL or host by resolving DNS and correlating it with Azure public IPs.
#[derive(Args, Debug, Clone)]
pub struct OutageInvestigateArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[arg(long, default_value_t)]
    pub tenant: AzureTenantArgument<'static>,

    /// Directory to write JSON blobs for relevant investigation artifacts.
    #[arg(long)]
    pub output_dir: Option<PathBuf>,

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
        let matches =
            enrich_matches_with_application_gateway_backend_health(tenant_id, matches).await?;
        let matches =
            enrich_matches_with_failing_backend_resource_discovery(tenant_id, matches).await?;

        let report = OutageInvestigationReport {
            input: self.target,
            tenant: tenant_id.to_string(),
            target_host,
            dns,
            matches,
        };

        if let Some(output_dir) = self.output_dir.as_deref() {
            write_investigation_artifacts(output_dir, &report)?;
        }

        print_pretty_report(&report, self.output_dir.as_deref());
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
    application_gateway_backend_health:
        Option<AzureApplicationGatewayResourceBackendHealthResponse>,
    application_gateway_backend_health_error: Option<String>,
    failing_backend_probe_investigations: Vec<FailingBackendProbeInvestigation>,
}

#[derive(Debug, Serialize, Clone)]
struct FailingBackendProbeInvestigation {
    backend_address_pool_id: String,
    backend_http_settings_id: String,
    server: AzureApplicationGatewayResourceBackendHealthServer,
    matching_network_interfaces: Vec<AzureNetworkInterfaceResource>,
    matching_private_endpoints: Vec<AzurePrivateEndpointResource>,
    private_link_service_ids: Vec<String>,
}

#[derive(Debug, Clone)]
struct ApplicationGatewayBackendHealthLookup {
    backend_health: Option<AzureApplicationGatewayResourceBackendHealthResponse>,
    error: Option<String>,
}

#[derive(Debug, Clone)]
struct FailingBackendServerCandidate {
    backend_address_pool_id: String,
    backend_http_settings_id: String,
    server: AzureApplicationGatewayResourceBackendHealthServer,
}

#[derive(Debug, Serialize)]
struct ApplicationGatewayBackendHealthArtifact {
    application_gateway_id: String,
    backend_health: Option<AzureApplicationGatewayResourceBackendHealthResponse>,
    error: Option<String>,
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
        application_gateway_backend_health: None,
        application_gateway_backend_health_error: None,
        failing_backend_probe_investigations: Vec::new(),
    })
}

async fn enrich_matches_with_application_gateway_backend_health(
    tenant_id: cloud_terrastodon_azure::AzureTenantId,
    mut matches: Vec<OutagePublicIpMatch>,
) -> Result<Vec<OutagePublicIpMatch>> {
    let backend_health_by_gateway_id =
        fetch_application_gateway_backend_health_for_matches(tenant_id, &matches).await;

    for matched_public_ip in &mut matches {
        let Some(application_gateway_id) = matched_public_ip.application_gateway_id.as_deref()
        else {
            continue;
        };

        let Some(lookup) = backend_health_by_gateway_id.get(application_gateway_id) else {
            continue;
        };

        matched_public_ip.application_gateway_backend_health = lookup.backend_health.clone();
        matched_public_ip.application_gateway_backend_health_error = lookup.error.clone();
    }

    Ok(matches)
}

async fn enrich_matches_with_failing_backend_resource_discovery(
    tenant_id: cloud_terrastodon_azure::AzureTenantId,
    mut matches: Vec<OutagePublicIpMatch>,
) -> Result<Vec<OutagePublicIpMatch>> {
    let failing_backend_candidates = matches
        .iter()
        .flat_map(|matched_public_ip| {
            matched_public_ip
                .application_gateway_backend_health
                .as_ref()
                .map(collect_failing_backend_servers)
                .unwrap_or_default()
        })
        .collect::<Vec<_>>();

    if failing_backend_candidates.is_empty() {
        return Ok(matches);
    }

    info!(
        count = failing_backend_candidates.len(),
        "Fetching network interfaces for failing backend probe investigation"
    );
    let network_interfaces = fetch_all_network_interfaces(tenant_id).await?;

    let relevant_private_endpoint_ids = failing_backend_candidates
        .iter()
        .flat_map(|candidate| {
            network_interfaces_for_backend_address(&network_interfaces, &candidate.server.address)
                .into_iter()
                .filter_map(|network_interface| managed_by_private_endpoint_id(&network_interface))
        })
        .collect::<BTreeSet<_>>();

    let private_endpoints_by_id = if relevant_private_endpoint_ids.is_empty() {
        BTreeMap::new()
    } else {
        info!(
            count = relevant_private_endpoint_ids.len(),
            "Fetching private endpoints for failing backend probe investigation"
        );
        fetch_all_private_endpoints(tenant_id)
            .await?
            .into_iter()
            .filter(|private_endpoint| {
                relevant_private_endpoint_ids.contains(&private_endpoint.id.expanded_form())
            })
            .map(|private_endpoint| (private_endpoint.id.expanded_form(), private_endpoint))
            .collect::<BTreeMap<_, _>>()
    };

    for matched_public_ip in &mut matches {
        let Some(backend_health) = matched_public_ip
            .application_gateway_backend_health
            .as_ref()
        else {
            continue;
        };

        matched_public_ip.failing_backend_probe_investigations =
            collect_failing_backend_servers(backend_health)
                .into_iter()
                .map(|candidate| {
                    let matching_network_interfaces = network_interfaces_for_backend_address(
                        &network_interfaces,
                        &candidate.server.address,
                    );
                    let matching_private_endpoints = matching_network_interfaces
                        .iter()
                        .filter_map(|network_interface| {
                            managed_by_private_endpoint_id(network_interface).and_then(
                                |private_endpoint_id| {
                                    private_endpoints_by_id.get(&private_endpoint_id).cloned()
                                },
                            )
                        })
                        .map(|private_endpoint| {
                            (private_endpoint.id.expanded_form(), private_endpoint)
                        })
                        .collect::<BTreeMap<_, _>>()
                        .into_values()
                        .collect::<Vec<_>>();
                    let private_link_service_ids = matching_private_endpoints
                        .iter()
                        .flat_map(private_link_service_ids)
                        .collect::<BTreeSet<_>>()
                        .into_iter()
                        .collect::<Vec<_>>();

                    FailingBackendProbeInvestigation {
                        backend_address_pool_id: candidate.backend_address_pool_id,
                        backend_http_settings_id: candidate.backend_http_settings_id,
                        server: candidate.server,
                        matching_network_interfaces,
                        matching_private_endpoints,
                        private_link_service_ids,
                    }
                })
                .collect();
    }

    Ok(matches)
}

async fn fetch_application_gateway_backend_health_for_matches(
    tenant_id: cloud_terrastodon_azure::AzureTenantId,
    matches: &[OutagePublicIpMatch],
) -> BTreeMap<String, ApplicationGatewayBackendHealthLookup> {
    let application_gateway_ids = matches
        .iter()
        .filter_map(|matched_public_ip| matched_public_ip.application_gateway_id.as_deref())
        .collect::<BTreeSet<_>>();

    if application_gateway_ids.is_empty() {
        return BTreeMap::new();
    }

    info!(
        count = application_gateway_ids.len(),
        "Fetching Application Gateway backend health for outage investigation"
    );

    let mut backend_health_by_gateway_id = BTreeMap::new();
    for application_gateway_id in application_gateway_ids {
        let lookup = match application_gateway_id.parse::<AzureApplicationGatewayResourceId>() {
            Ok(application_gateway_id) => {
                match fetch_application_gateway_backend_health(tenant_id, application_gateway_id)
                    .await
                {
                    Ok(backend_health) => ApplicationGatewayBackendHealthLookup {
                        backend_health: Some(backend_health),
                        error: None,
                    },
                    Err(error) => ApplicationGatewayBackendHealthLookup {
                        backend_health: None,
                        error: Some(error.to_string()),
                    },
                }
            }
            Err(error) => ApplicationGatewayBackendHealthLookup {
                backend_health: None,
                error: Some(format!(
                    "Failed to parse application gateway id '{}': {}",
                    application_gateway_id, error
                )),
            },
        };

        backend_health_by_gateway_id.insert(application_gateway_id.to_string(), lookup);
    }

    backend_health_by_gateway_id
}

fn application_gateway_id_from_frontend_ip_configuration_id(id: &str) -> Option<String> {
    let marker = "/frontendIPConfigurations/";
    let lower = id.to_ascii_lowercase();
    let marker_index = lower.find(&marker.to_ascii_lowercase())?;
    Some(id[..marker_index].to_string())
}

fn collect_failing_backend_servers(
    backend_health: &AzureApplicationGatewayResourceBackendHealthResponse,
) -> Vec<FailingBackendServerCandidate> {
    backend_health
        .backend_address_pools
        .iter()
        .flat_map(|pool| {
            pool.backend_http_settings_collection
                .iter()
                .flat_map(|http_settings| {
                    http_settings
                        .servers
                        .iter()
                        .filter(|server| is_failing_backend_server(server))
                        .map(|server| FailingBackendServerCandidate {
                            backend_address_pool_id: pool.backend_address_pool.id.clone(),
                            backend_http_settings_id: http_settings
                                .backend_http_settings
                                .id
                                .clone(),
                            server: server.clone(),
                        })
                })
        })
        .collect()
}

fn is_failing_backend_server(server: &AzureApplicationGatewayResourceBackendHealthServer) -> bool {
    server.health_probe_log.is_some()
        || server.health_probe_error_name.is_some()
        || !matches!(
            server.health,
            AzureApplicationGatewayResourceBackendHealthServerHealth::Healthy
                | AzureApplicationGatewayResourceBackendHealthServerHealth::Up
        )
}

fn network_interfaces_for_backend_address(
    network_interfaces: &[AzureNetworkInterfaceResource],
    backend_address: &str,
) -> Vec<AzureNetworkInterfaceResource> {
    network_interfaces
        .iter()
        .filter(|network_interface| {
            network_interface_matches_backend_address(network_interface, backend_address)
        })
        .cloned()
        .collect()
}

fn network_interface_matches_backend_address(
    network_interface: &AzureNetworkInterfaceResource,
    backend_address: &str,
) -> bool {
    let normalized_address = normalize_host(backend_address);
    network_interface
        .properties
        .ip_configurations
        .iter()
        .any(|ip_configuration| {
            ip_configuration
                .properties
                .private_ip_address
                .map(|ip| ip.to_string() == backend_address)
                .unwrap_or(false)
        })
        || network_interface
            .properties
            .dns_settings
            .as_ref()
            .and_then(|dns_settings| dns_settings.internal_fqdn.as_deref())
            .map(normalize_host)
            .map(|fqdn| fqdn == normalized_address)
            .unwrap_or(false)
}

fn managed_by_private_endpoint_id(
    network_interface: &AzureNetworkInterfaceResource,
) -> Option<String> {
    let managed_by = network_interface.managed_by.as_deref()?;
    managed_by
        .parse::<AzurePrivateEndpointResourceId>()
        .ok()
        .map(|private_endpoint_id| private_endpoint_id.expanded_form())
}

fn private_link_service_ids(private_endpoint: &AzurePrivateEndpointResource) -> Vec<String> {
    private_endpoint
        .properties
        .private_link_service_connections
        .iter()
        .chain(
            private_endpoint
                .properties
                .manual_private_link_service_connections
                .iter(),
        )
        .filter_map(|connection| connection.properties.private_link_service_id.clone())
        .collect()
}

fn print_pretty_report(report: &OutageInvestigationReport, output_dir: Option<&Path>) {
    println!(
        "{} {}",
        "Outage investigation".bold().bright_blue(),
        report.input.bold()
    );
    println!("{} {}", "Tenant:".bold(), report.tenant.cyan());
    println!("{} {}", "Target host:".bold(), report.target_host.cyan());
    println!();

    println!("{}", "DNS".bold().bright_blue());
    println!(
        "  {} {}",
        "Canonical:".bold(),
        report.dns.canonical_name.as_str().green()
    );
    if report.dns.aliases.is_empty() {
        println!("  {} {}", "Aliases:".bold(), "none".dimmed());
    } else {
        println!(
            "  {} {}",
            "Aliases:".bold(),
            report.dns.aliases.join(", ").yellow()
        );
    }
    println!(
        "  {} {}",
        "Addresses:".bold(),
        report
            .dns
            .addresses
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ")
            .magenta()
    );
    println!();

    if report.matches.is_empty() {
        println!(
            "{}",
            "No matching Azure public IP resources found."
                .yellow()
                .bold()
        );
    } else {
        println!(
            "{} {}",
            "Public IP matches:".bold().bright_blue(),
            report.matches.len()
        );
        for matched_public_ip in &report.matches {
            let public_ip_name = matched_public_ip.public_ip.name.to_string();
            let public_ip_value = matched_public_ip
                .public_ip
                .properties
                .ip_address
                .map(|ip| ip.to_string())
                .unwrap_or_else(|| "<no ip>".to_string());
            println!(
                "  {} {} {}",
                "Public IP".bold(),
                public_ip_name.green(),
                format!("({})", public_ip_value).dimmed()
            );
            println!(
                "    {} {}",
                "Resource ID:".bold(),
                matched_public_ip.public_ip.id.expanded_form().dimmed()
            );

            match matched_public_ip.application_gateway_id.as_deref() {
                Some(application_gateway_id) => println!(
                    "    {} {}",
                    "Application Gateway:".bold(),
                    application_gateway_id.cyan()
                ),
                None => println!("    {} {}", "Application Gateway:".bold(), "none".dimmed()),
            }

            if let Some(error) = matched_public_ip
                .application_gateway_backend_health_error
                .as_deref()
            {
                println!("    {} {}", "Backend health error:".bold(), error.red());
            }

            if matched_public_ip
                .failing_backend_probe_investigations
                .is_empty()
            {
                println!("    {}", "No failing backend probes discovered.".green());
                continue;
            }

            println!(
                "    {} {}",
                "Failing backend probes:".bold().red(),
                matched_public_ip.failing_backend_probe_investigations.len()
            );
            for investigation in &matched_public_ip.failing_backend_probe_investigations {
                let health: String = investigation.server.health.clone().into();
                println!(
                    "      {} {} {}",
                    "Backend".bold(),
                    investigation.server.address.yellow(),
                    format!("[{}]", health).red()
                );
                println!(
                    "        {} {}",
                    "Pool:".bold(),
                    investigation.backend_address_pool_id.dimmed()
                );
                println!(
                    "        {} {}",
                    "HTTP settings:".bold(),
                    investigation.backend_http_settings_id.dimmed()
                );
                if let Some(probe_log) = investigation.server.health_probe_log.as_deref() {
                    println!("        {} {}", "Probe log:".bold(), probe_log.red());
                }
                if let Some(probe_error_name) = investigation.server.health_probe_error_name.clone()
                {
                    let probe_error: String = String::from(probe_error_name);
                    println!("        {} {}", "Probe error:".bold(), probe_error.red());
                }

                if investigation.matching_network_interfaces.is_empty() {
                    println!(
                        "        {}",
                        "No matching network interfaces found.".yellow()
                    );
                } else {
                    for network_interface in &investigation.matching_network_interfaces {
                        println!(
                            "        {} {}",
                            "NIC:".bold(),
                            network_interface.name.to_string().green()
                        );
                        println!(
                            "          {} {}",
                            "Resource ID:".bold(),
                            network_interface.id.expanded_form().dimmed()
                        );
                        match network_interface.managed_by.as_deref() {
                            Some(managed_by) => {
                                println!("          {} {}", "managedBy:".bold(), managed_by.cyan())
                            }
                            None => {
                                println!("          {} {}", "managedBy:".bold(), "none".dimmed())
                            }
                        }
                    }
                }

                if investigation.matching_private_endpoints.is_empty() {
                    println!(
                        "        {}",
                        "No matching private endpoints found from NIC managedBy.".yellow()
                    );
                } else {
                    for private_endpoint in &investigation.matching_private_endpoints {
                        println!(
                            "        {} {}",
                            "Private endpoint:".bold(),
                            private_endpoint.name.to_string().green()
                        );
                        println!(
                            "          {} {}",
                            "Resource ID:".bold(),
                            private_endpoint.id.expanded_form().dimmed()
                        );
                    }
                }

                if investigation.private_link_service_ids.is_empty() {
                    println!(
                        "        {} {}",
                        "PrivateLinkServiceId:".bold(),
                        "none".dimmed()
                    );
                } else {
                    for private_link_service_id in &investigation.private_link_service_ids {
                        println!(
                            "        {} {}",
                            "PrivateLinkServiceId:".bold(),
                            private_link_service_id.magenta()
                        );
                    }
                }
            }
        }
    }

    if let Some(output_dir) = output_dir {
        println!();
        println!(
            "{} {}",
            "JSON artifacts written to".bold().bright_blue(),
            output_dir.display().to_string().cyan()
        );
    }
}

fn write_investigation_artifacts(
    output_dir: &Path,
    report: &OutageInvestigationReport,
) -> Result<()> {
    std::fs::create_dir_all(output_dir)
        .with_context(|| format!("creating output dir '{}'", output_dir.display()))?;

    write_json_file(output_dir.join("report.json"), report)?;

    let public_ips = report
        .matches
        .iter()
        .map(|matched_public_ip| matched_public_ip.public_ip.clone())
        .collect::<Vec<_>>();
    write_json_file(output_dir.join("matched-public-ips.json"), &public_ips)?;

    let backend_health = report
        .matches
        .iter()
        .map(
            |matched_public_ip| ApplicationGatewayBackendHealthArtifact {
                application_gateway_id: matched_public_ip
                    .application_gateway_id
                    .clone()
                    .unwrap_or_default(),
                backend_health: matched_public_ip.application_gateway_backend_health.clone(),
                error: matched_public_ip
                    .application_gateway_backend_health_error
                    .clone(),
            },
        )
        .collect::<Vec<_>>();
    write_json_file(
        output_dir.join("application-gateway-backend-health.json"),
        &backend_health,
    )?;

    let backend_probe_investigations = report
        .matches
        .iter()
        .flat_map(|matched_public_ip| {
            matched_public_ip
                .failing_backend_probe_investigations
                .iter()
                .cloned()
        })
        .collect::<Vec<_>>();
    write_json_file(
        output_dir.join("failing-backend-probe-investigations.json"),
        &backend_probe_investigations,
    )?;

    let network_interfaces = report
        .matches
        .iter()
        .flat_map(|matched_public_ip| {
            matched_public_ip
                .failing_backend_probe_investigations
                .iter()
                .flat_map(|investigation| investigation.matching_network_interfaces.iter().cloned())
        })
        .map(|network_interface| (network_interface.id.expanded_form(), network_interface))
        .collect::<BTreeMap<_, _>>()
        .into_values()
        .collect::<Vec<_>>();
    write_json_file(
        output_dir.join("matching-network-interfaces.json"),
        &network_interfaces,
    )?;

    let private_endpoints = report
        .matches
        .iter()
        .flat_map(|matched_public_ip| {
            matched_public_ip
                .failing_backend_probe_investigations
                .iter()
                .flat_map(|investigation| investigation.matching_private_endpoints.iter().cloned())
        })
        .map(|private_endpoint| (private_endpoint.id.expanded_form(), private_endpoint))
        .collect::<BTreeMap<_, _>>()
        .into_values()
        .collect::<Vec<_>>();
    write_json_file(
        output_dir.join("matching-private-endpoints.json"),
        &private_endpoints,
    )?;

    Ok(())
}

fn write_json_file(path: PathBuf, value: &impl Serialize) -> Result<()> {
    let file =
        std::fs::File::create(&path).with_context(|| format!("creating '{}'", path.display()))?;
    serde_json::to_writer_pretty(file, value)
        .with_context(|| format!("writing '{}'", path.display()))?;
    Ok(())
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

    #[test]
    fn non_gateway_ids_do_not_derive_application_gateway_id() {
        let id = "/subscriptions/123/resourceGroups/rg/providers/Microsoft.Network/publicIPAddresses/pip";
        assert_eq!(
            application_gateway_id_from_frontend_ip_configuration_id(id),
            None
        );
    }

    #[test]
    fn network_interface_backend_address_match_supports_private_ip_and_internal_fqdn() -> Result<()>
    {
        let network_interface = serde_json::from_str::<AzureNetworkInterfaceResource>(
            r#"
                        {
                            "id": "/subscriptions/11111111-1111-1111-1111-111111111111/resourceGroups/my-rg/providers/Microsoft.Network/networkInterfaces/my-nic",
                            "tenantId": "22222222-2222-2222-2222-222222222222",
                            "name": "my-nic",
                            "location": "canadacentral",
                            "managedBy": "/subscriptions/11111111-1111-1111-1111-111111111111/resourceGroups/my-rg/providers/Microsoft.Network/privateEndpoints/my-pe",
                            "tags": {},
                            "properties": {
                                "dnsSettings": {
                                    "internalFqdn": "my-nic.internal.example"
                                },
                                "ipConfigurations": [
                                    {
                                        "name": "ipconfig1",
                                        "id": "/subscriptions/11111111-1111-1111-1111-111111111111/resourceGroups/my-rg/providers/Microsoft.Network/networkInterfaces/my-nic/ipConfigurations/ipconfig1",
                                        "properties": {
                                            "privateIPAddress": "10.0.0.5"
                                        }
                                    }
                                ]
                            }
                        }
                        "#,
        )?;

        assert!(network_interface_matches_backend_address(
            &network_interface,
            "10.0.0.5"
        ));
        assert!(network_interface_matches_backend_address(
            &network_interface,
            "my-nic.internal.example"
        ));
        assert_eq!(
            managed_by_private_endpoint_id(&network_interface).as_deref(),
            Some(
                "/subscriptions/11111111-1111-1111-1111-111111111111/resourceGroups/my-rg/providers/Microsoft.Network/privateEndpoints/my-pe"
            )
        );
        Ok(())
    }
}
