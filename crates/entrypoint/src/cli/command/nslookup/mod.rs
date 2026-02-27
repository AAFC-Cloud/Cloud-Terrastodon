use clap::Args;
use eyre::Context;
use eyre::Result;
use eyre::bail;
use hickory_resolver::config::ResolverConfig;
use hickory_resolver::config::ResolverOpts;
use std::collections::BTreeSet;
use std::collections::HashSet;
use std::net::IpAddr;
use hickory_resolver::TokioResolver;
use hickory_resolver::proto::rr::RData;
use hickory_resolver::proto::rr::RecordType;
use hickory_resolver::system_conf::read_system_conf;

/// Resolve a host name to IP addresses.
#[derive(Args, Debug, Clone)]
pub struct NslookupArgs {
    /// Host name or IP address to resolve.
    pub target: String,
}

impl NslookupArgs {
    pub async fn invoke(self) -> Result<()> {
        let target = self.target.trim();
        if target.is_empty() {
            bail!("target cannot be empty");
        }

        let (resolver, resolver_server) = build_system_resolver().await?;

        if let Some(server) = resolver_server {
            println!("Server:\t{server}");
            println!();
        }

        if let Ok(ip) = target.parse::<IpAddr>() {
            if let Ok(reverse) = resolver.reverse_lookup(ip).await {
                if let Some(name) = reverse.iter().next() {
                    println!("Name:\t{}", trim_fqdn_dot(name.to_utf8()));
                }
            }
            println!("Address:\t{ip}");
            return Ok(());
        }

        let (canonical_name, aliases) = resolve_cname_chain(&resolver, target).await?;

        let lookup = resolver
            .lookup_ip(canonical_name.clone())
            .await
            .with_context(|| format!("failed to resolve '{target}'"))?;

        let unique_ips = lookup.iter().collect::<BTreeSet<_>>();
        if unique_ips.is_empty() {
            bail!("no addresses were found for '{target}'");
        }

        println!("Name:\t{canonical_name}");
        for ip in unique_ips {
            println!("Address:\t{ip}");
        }
        if !aliases.is_empty() {
            let mut aliases_iter = aliases.into_iter();
            if let Some(first) = aliases_iter.next() {
                println!("Aliases:\t{first}");
            }
            for alias in aliases_iter {
                println!("\t\t{alias}");
            }
        }

        Ok(())
    }
}

async fn build_system_resolver() -> Result<(TokioResolver, Option<String>)> {
    let (config, opts): (ResolverConfig, ResolverOpts) = read_system_conf()
        .context("reading system DNS configuration")?;

    let server_ip = config.name_servers().first().map(|ns| ns.socket_addr.ip());

    let resolver = TokioResolver::builder_with_config(config, Default::default())
        .with_options(opts)
        .build();

    let server_display = if let Some(ip) = server_ip {
        let hostname = resolver
            .reverse_lookup(ip)
            .await
            .ok()
            .and_then(|lookup| lookup.iter().next().map(|name| trim_fqdn_dot(name.to_utf8())));
        Some(match hostname {
            Some(hostname) => format!("{hostname}\nAddress:\t{ip}"),
            None => format!("{ip}"),
        })
    } else {
        None
    };

    Ok((resolver, server_display))
}

async fn resolve_cname_chain(resolver: &TokioResolver, query: &str) -> Result<(String, Vec<String>)> {
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
    aliases.retain(|x| x != &canonical);
    Ok((canonical, aliases))
}

fn trim_fqdn_dot(name: String) -> String {
    name.trim_end_matches('.').to_string()
}
