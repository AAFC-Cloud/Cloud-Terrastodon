use super::error_summary;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_azure::AzureTenantAlias;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::AzureTenantId;
use cloud_terrastodon_azure::Subscription;
use cloud_terrastodon_azure::fetch_all_subscriptions;
use cloud_terrastodon_azure::fetch_azure_tenant_details;
use cloud_terrastodon_azure::get_tracked_tenant;
use cloud_terrastodon_azure::list_tracked_tenant_aliases_for;
use cloud_terrastodon_azure::list_tracked_tenants;
use cloud_terrastodon_azure::resolve_tenant_auth_context;
use cloud_terrastodon_credentials::AuthContext;
use color_eyre::owo_colors::OwoColorize;
use eyre::Result;
use std::fmt::Write;
use std::future::Future;
use tokio::task::JoinSet;
use tracing::debug;

const MAX_CONCURRENT_TENANTS: usize = 4;

/// Arguments for listing Azure subscriptions across tracked tenants.
#[derive(facet::Facet, Debug, Clone, Default)]
pub struct AzureSubscriptionListArgs {
    /// Optional tenant id or tracked alias to query. Defaults to all tracked tenants.
    #[facet(figue::named)]
    pub tenant: Option<AzureTenantArgument<'static>>,
}

#[derive(Debug, facet::Facet)]
struct TenantSubscriptions {
    tenant_id: AzureTenantId,
    tenant_name: Option<String>,
    tenant_aliases: Vec<AzureTenantAlias>,
    subscriptions: Vec<ListedSubscription>,
    /// Concise subscription/authentication failure; empty successful tenants have no error.
    error: Option<String>,
    /// Name/alias lookups may fail without preventing subscription enumeration.
    metadata_errors: Vec<String>,
}

#[derive(Debug, facet::Facet)]
struct ListedSubscription {
    #[facet(flatten)]
    subscription: Subscription,
    portal_url: String,
}

#[derive(Debug, facet::Facet)]
#[facet(transparent)]
struct SubscriptionListOutput(Vec<TenantSubscriptions>);

impl AzureSubscriptionListArgs {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<CliOutput> {
        let tenant_ids = match self.tenant {
            Some(tenant) => {
                let tenant_id = match (&tenant, auth_context.tenant_id()) {
                    (AzureTenantArgument::Default, Some(tenant_id)) => tenant_id,
                    _ => tenant.resolve().await?,
                };
                vec![tenant_id]
            }
            None => list_tracked_tenants().await?,
        };
        debug!(
            count = tenant_ids.len(),
            "Fetching subscriptions across Azure tenants"
        );
        let tenants = collect_tenants(tenant_ids, |tenant_id| {
            let auth_context = auth_context.clone();
            async move { fetch_tenant_subscriptions(tenant_id, &auth_context).await }
        })
        .await?;
        Ok(SubscriptionListOutput(tenants).into_cli_output())
    }
}

/// Keep tenant requests bounded and preserve a deterministic order regardless
/// of which tenant finishes first.
async fn collect_tenants<F, Fut>(
    tenant_ids: Vec<AzureTenantId>,
    fetch: F,
) -> Result<Vec<TenantSubscriptions>>
where
    F: Fn(AzureTenantId) -> Fut,
    Fut: Future<Output = TenantSubscriptions> + Send + 'static,
{
    let mut pending = tenant_ids.into_iter();
    let mut requests = JoinSet::new();
    for tenant_id in pending.by_ref().take(MAX_CONCURRENT_TENANTS) {
        requests.spawn(fetch(tenant_id));
    }
    let mut tenants = Vec::new();
    while let Some(result) = requests.join_next().await {
        tenants.push(result?);
        if let Some(tenant_id) = pending.next() {
            requests.spawn(fetch(tenant_id));
        }
    }
    tenants.sort_by_key(|tenant| tenant.tenant_id);
    Ok(tenants)
}

async fn fetch_tenant_subscriptions(
    tenant_id: AzureTenantId,
    auth_context: &AuthContext,
) -> TenantSubscriptions {
    let aliases = async {
        // An explicit `--tenant default` may select an untracked Azure CLI tenant.
        if get_tracked_tenant(tenant_id).await?.is_none() {
            return Ok(Vec::new());
        }
        list_tracked_tenant_aliases_for(tenant_id).await
    };
    let binding = async {
        let auth_context = resolve_tenant_auth_context(auth_context, tenant_id).await?;
        auth_context.bind_to_azure_tenant(tenant_id)
    }
    .await;
    let tenant_auth_context = match binding {
        Ok(context) => context,
        Err(error) => {
            return tenant_output(tenant_id, aliases.await, Ok(None), Err(error));
        }
    };
    let (aliases, details, subscriptions) = tokio::join!(
        aliases,
        async {
            fetch_azure_tenant_details(&tenant_auth_context)
                .await
                .map(|details| Some(details.display_name))
        },
        async { fetch_all_subscriptions(&tenant_auth_context).await },
    );
    tenant_output(tenant_id, aliases, details, subscriptions)
}

fn tenant_output(
    tenant_id: AzureTenantId,
    aliases: Result<Vec<AzureTenantAlias>>,
    name: Result<Option<String>>,
    subscriptions: Result<Vec<Subscription>>,
) -> TenantSubscriptions {
    let mut metadata_errors = Vec::new();
    let mut tenant_aliases = match aliases {
        Ok(aliases) => aliases,
        Err(error) => {
            debug!(error = ?error, %tenant_id, "Tenant aliases lookup failed");
            metadata_errors.push(format!(
                "Tenant aliases unavailable: {}",
                error_summary::summarize(&error)
            ));
            Vec::new()
        }
    };
    tenant_aliases.sort();
    tenant_aliases.dedup();
    let tenant_name = match name {
        Ok(name) => name,
        Err(error) => {
            debug!(error = ?error, %tenant_id, "Tenant name lookup failed");
            metadata_errors.push(format!(
                "Tenant name unavailable: {}",
                error_summary::summarize(&error)
            ));
            None
        }
    };
    let (mut subscriptions, error) = match subscriptions {
        Ok(subscriptions) => (subscriptions, None),
        Err(error) => {
            debug!(error = ?error, %tenant_id, "Subscription lookup failed");
            (Vec::new(), Some(error_summary::summarize(&error)))
        }
    };
    subscriptions.sort_by(|left, right| {
        left.name
            .to_ascii_lowercase()
            .cmp(&right.name.to_ascii_lowercase())
            .then_with(|| left.id.cmp(&right.id))
    });
    let subscriptions = subscriptions
        .into_iter()
        .map(|subscription| ListedSubscription {
            portal_url: format!(
                "https://portal.azure.com/#@{tenant_id}/resource/subscriptions/{}/overview",
                subscription.id
            ),
            subscription,
        })
        .collect();
    TenantSubscriptions {
        tenant_id,
        tenant_name,
        tenant_aliases,
        subscriptions,
        error,
        metadata_errors,
    }
}

impl SubscriptionListOutput {
    fn into_cli_output(self) -> CliOutput {
        CliOutput::facet_with_text(self, Self::render_text)
    }

    fn render_text(&self, terminal: bool) -> Result<String> {
        let mut output = String::new();
        if self.0.is_empty() {
            output.push_str("No tracked Azure tenants. Add one with `cloud_terrastodon az tenant add <tenant-id>` or discover them with `cloud_terrastodon az tenant discover`.\n");
            return Ok(output);
        }
        for (index, tenant) in self.0.iter().enumerate() {
            if index > 0 {
                output.push('\n');
            }
            let name = terminal_text(
                tenant
                    .tenant_name
                    .as_deref()
                    .unwrap_or("(name unavailable)"),
            );
            let id = tenant.tenant_id.to_string();
            let aliases = if tenant.tenant_aliases.is_empty() {
                "(none)".to_owned()
            } else {
                tenant
                    .tenant_aliases
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            if terminal {
                writeln!(
                    output,
                    "{} {}  {}  aliases: {}",
                    "Tenant".cyan().bold(),
                    name.cyan().bold(),
                    id.dimmed(),
                    aliases.magenta()
                )?;
            } else {
                writeln!(output, "Tenant {name}  {id}  aliases: {aliases}")?;
            }
            for error in &tenant.metadata_errors {
                // The name and subscriptions may both fail on the same token.
                // The header already identifies the missing name.
                if error.strip_prefix("Tenant name unavailable: ") == tenant.error.as_deref()
                    && tenant.error.is_some()
                {
                    continue;
                }
                let error = terminal_text(error);
                let error = if terminal {
                    error.yellow().to_string()
                } else {
                    error
                };
                writeln!(output, "  │  {error}")?;
            }
            if let Some(error) = &tenant.error {
                let message = format!("Unable to list subscriptions: {}", terminal_text(error));
                let message = if terminal {
                    message.yellow().to_string()
                } else {
                    message
                };
                writeln!(output, "  └─ {message}")?;
            } else if tenant.subscriptions.is_empty() {
                writeln!(output, "  └─ No subscriptions found.")?;
            } else {
                for (index, listed) in tenant.subscriptions.iter().enumerate() {
                    let branch = if index + 1 == tenant.subscriptions.len() {
                        "└─"
                    } else {
                        "├─"
                    };
                    let name = terminal_text(&listed.subscription.name);
                    let id = listed.subscription.id.to_string();
                    let link = if terminal {
                        terminal_link("Azure portal", &listed.portal_url)
                            .bright_blue()
                            .underline()
                            .to_string()
                    } else {
                        listed.portal_url.clone()
                    };
                    if terminal {
                        writeln!(
                            output,
                            "  {} {}  {}  {}",
                            branch.dimmed(),
                            id.green(),
                            name.bold(),
                            link
                        )?;
                    } else {
                        writeln!(output, "  {branch} {id}  {name}  {link}")?;
                    }
                }
            }
        }
        Ok(output)
    }
}

/// Cloud names and errors are untrusted terminal text, including in OSC labels.
fn terminal_text(text: &str) -> String {
    text.chars()
        .map(|ch| if ch.is_control() { ' ' } else { ch })
        .collect()
}

fn terminal_link(label: &str, url: &str) -> String {
    format!("\x1b]8;;{url}\x1b\\{label}\x1b]8;;\x1b\\")
}

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_terrastodon_app::OutputFormat;

    fn subscription(tenant_id: AzureTenantId, name: &str, id: &str) -> Subscription {
        facet_json::from_str(&format!(
            r#"{{"id":"{id}","name":"{name}","tenant_id":"{tenant_id}","management_group_ancestors_chain":[],"tags":{{"team":"cloud"}}}}"#
        )).unwrap()
    }

    fn example_output() -> SubscriptionListOutput {
        let tenant_id = "11111111-1111-1111-1111-111111111111".parse().unwrap();
        let subscriptions = vec![subscription(
            tenant_id,
            "Production",
            "22222222-2222-2222-2222-222222222222",
        )];
        SubscriptionListOutput(vec![tenant_output(
            tenant_id,
            Ok(vec!["prod".parse().unwrap(), "cloud".parse().unwrap()]),
            Ok(Some("Contoso".to_owned())),
            Ok(subscriptions),
        )])
    }

    #[derive(Debug, facet::Facet)]
    struct ParseArgs {
        #[facet(flatten, default)]
        args: AzureSubscriptionListArgs,
    }

    fn parse_args(arguments: &[&str]) -> ParseArgs {
        let config = figue::builder::<ParseArgs>()
            .unwrap()
            .cli(|cli| {
                cli.args_os(
                    arguments
                        .iter()
                        .map(|argument| std::ffi::OsString::from(*argument)),
                )
                .strict()
            })
            .build();
        figue::Driver::new(config)
            .run()
            .into_result()
            .unwrap()
            .value
    }

    #[test]
    fn omitted_tenant_selects_all_and_explicit_tenant_preserves_alias_selector() {
        let all = parse_args(&[]);
        assert!(all.args.tenant.is_none());
        let filtered = parse_args(&["--tenant", "prod"]);
        assert_eq!(
            filtered
                .args
                .tenant
                .map(|tenant| tenant.to_string())
                .as_deref(),
            Some("prod")
        );
    }

    #[test]
    fn terminal_text_shows_tenant_metadata_subscriptions_and_clickable_portal_link() {
        let output = example_output()
            .into_cli_output()
            .render(Some(OutputFormat::Text), true)
            .unwrap()
            .unwrap();
        assert!(output.contains("Contoso"));
        assert!(output.contains("11111111-1111-1111-1111-111111111111"));
        assert!(output.contains("cloud, prod"));
        assert!(output.contains("22222222-2222-2222-2222-222222222222"));
        assert!(output.contains("Production"));
        assert!(output.contains("\x1b["));
        assert!(output.contains("\x1b]8;;https://portal.azure.com/#@11111111-1111-1111-1111-111111111111/resource/subscriptions/22222222-2222-2222-2222-222222222222/overview\x1b\\"));
    }

    #[test]
    fn piped_text_has_visible_portal_url_without_terminal_escapes() {
        let output = example_output()
            .into_cli_output()
            .render(Some(OutputFormat::Text), false)
            .unwrap()
            .unwrap();
        assert!(output.contains("https://portal.azure.com/#@"));
        assert!(!output.contains('\x1b'));
    }

    #[test]
    fn facet_pretty_shows_underlying_tenant_data_without_custom_terminal_links() {
        for terminal in [true, false] {
            let output = example_output()
                .into_cli_output()
                .render(Some(OutputFormat::FacetPretty), terminal)
                .unwrap()
                .unwrap();
            assert!(output.contains("tenant_name"), "{output}");
            assert!(output.contains("tenant_aliases"), "{output}");
            assert!(output.contains("portal_url"), "{output}");
            assert!(output.contains("Contoso"), "{output}");
            assert!(output.contains("Production"), "{output}");
            assert!(!output.contains("\x1b]8;;"), "{output}");
            assert_eq!(output.contains('\x1b'), terminal, "{output}");
        }
    }

    #[test]
    fn json_keeps_subscription_details_tenant_metadata_and_portal_url() {
        let output = example_output()
            .into_cli_output()
            .render(Some(OutputFormat::Json), true)
            .unwrap()
            .unwrap();
        let output: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(output[0]["tenant_name"], "Contoso");
        assert_eq!(
            output[0]["tenant_aliases"],
            serde_json::json!(["cloud", "prod"])
        );
        assert!(output[0]["error"].is_null());
        assert_eq!(
            output[0]["subscriptions"][0]["id"],
            "22222222-2222-2222-2222-222222222222"
        );
        assert_eq!(output[0]["subscriptions"][0]["tags"]["team"], "cloud");
        assert!(
            output[0]["subscriptions"][0]["portal_url"]
                .as_str()
                .unwrap()
                .ends_with("/overview")
        );
    }

    #[test]
    fn name_or_alias_lookup_failure_does_not_hide_subscriptions() {
        let tenant_id = "11111111-1111-1111-1111-111111111111".parse().unwrap();
        let tenant = tenant_output(
            tenant_id,
            Err(eyre::eyre!("aliases file unreadable")),
            Err(eyre::eyre!("Graph access denied")),
            Ok(vec![subscription(
                tenant_id,
                "Production",
                "22222222-2222-2222-2222-222222222222",
            )]),
        );
        assert_eq!(tenant.subscriptions.len(), 1);
        assert!(tenant.error.is_none());
        assert!(tenant.tenant_name.is_none());
        assert_eq!(tenant.metadata_errors.len(), 2);
        let text = SubscriptionListOutput(vec![tenant])
            .render_text(false)
            .unwrap();
        assert!(text.contains("Tenant aliases unavailable: aliases file unreadable"));
        assert!(text.contains("Tenant name unavailable: Graph access denied"));
        assert!(text.contains("Production"));
    }

    #[test]
    fn repeated_cli_auth_failures_show_one_concise_message_and_keep_json_errors_usable() {
        fn cli_error(context: &str) -> eyre::Report {
            eyre::Report::new(cloud_terrastodon_command::CommandOutput {
                status: 1,
                stdout: "".into(),
                stderr: concat!(
                    "DEBUG: cli.knack.cli: synthetic command arguments\n",
                    "DEBUG: cli.azure.cli.core.azclierror: Traceback (most recent call last):\n",
                    "  File synthetic.py, line 42\n",
                    "ERROR: cli.azure.cli.core.azclierror: No subscription found. Run 'az account set' to select a subscription.\n",
                    "ERROR: az_command_data_logger: No subscription found. Run 'az account set' to select a subscription.\n",
                    "INFO: cli.__main__: synthetic command finished\n",
                ).into(),
            }).wrap_err(context.to_owned())
        }

        let tenant = tenant_output(
            "11111111-1111-1111-1111-111111111111".parse().unwrap(),
            Ok(vec!["example".parse().unwrap()]),
            Err(cli_error("Fetching synthetic Graph name failed")),
            Err(cli_error("Fetching synthetic ARM subscriptions failed")),
        );
        let output = SubscriptionListOutput(vec![tenant]).into_cli_output();
        let message = "No subscription found. Run 'az account set' to select a subscription.";
        for terminal in [true, false] {
            let text = output
                .render(Some(OutputFormat::Text), terminal)
                .unwrap()
                .unwrap();
            assert_eq!(text.matches(message).count(), 1);
            assert!(text.contains("Unable to list subscriptions:"));
            assert!(!text.contains("Tenant name unavailable:"));
            assert!(!text.contains("DEBUG:"));
            assert!(!text.contains("Traceback"));
            assert!(!text.contains("synthetic.py"));
            assert_eq!(text.lines().count(), 2);
        }
        let json = output
            .render(Some(OutputFormat::Json), false)
            .unwrap()
            .unwrap();
        let json: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(json[0]["error"], message);
        assert_eq!(
            json[0]["metadata_errors"][0],
            format!("Tenant name unavailable: {message}")
        );
    }

    #[test]
    fn distinct_metadata_errors_remain_visible_beside_a_subscription_failure() {
        let tenant = tenant_output(
            "11111111-1111-1111-1111-111111111111".parse().unwrap(),
            Err(eyre::eyre!("Local aliases are unavailable")),
            Err(eyre::eyre!("Graph permission denied").wrap_err("Fetching tenant name")),
            Err(eyre::eyre!("Subscription request was throttled").wrap_err("Querying ARM")),
        );
        let text = SubscriptionListOutput(vec![tenant])
            .render_text(false)
            .unwrap();
        assert!(text.contains("Tenant aliases unavailable: Local aliases are unavailable"));
        assert!(text.contains("Tenant name unavailable: Graph permission denied"));
        assert!(text.contains("Unable to list subscriptions: Subscription request was throttled"));
        assert!(!text.contains("Fetching tenant name"));
        assert!(!text.contains("Querying ARM"));
    }

    #[tokio::test]
    async fn failed_and_empty_tenants_survive_collection_beside_successful_tenants() {
        let ids: Vec<AzureTenantId> = [
            "33333333-3333-3333-3333-333333333333",
            "22222222-2222-2222-2222-222222222222",
            "11111111-1111-1111-1111-111111111111",
            "44444444-4444-4444-4444-444444444444",
            "55555555-5555-5555-5555-555555555555",
            "66666666-6666-6666-6666-666666666666",
            "77777777-7777-7777-7777-777777777777",
        ]
        .into_iter()
        .map(|id| id.parse().unwrap())
        .collect();
        let failed_id = ids[0];
        let empty_id = ids[1];
        let tenants = collect_tenants(ids, move |tenant_id| async move {
            let subscriptions = if tenant_id == failed_id {
                Err(eyre::eyre!("Authentication expired; run az login"))
            } else if tenant_id == empty_id {
                Ok(Vec::new())
            } else {
                Ok(vec![subscription(
                    tenant_id,
                    "Production",
                    "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
                )])
            };
            tenant_output(
                tenant_id,
                Ok(Vec::new()),
                Ok(Some("Contoso".to_owned())),
                subscriptions,
            )
        })
        .await
        .unwrap();
        assert_eq!(tenants.len(), 7);
        assert_eq!(tenants[0].subscriptions.len(), 1);
        assert_eq!(tenants[6].subscriptions.len(), 1);
        assert!(tenants[1].error.is_none());
        assert!(tenants[1].subscriptions.is_empty());
        assert!(
            tenants[2]
                .error
                .as_deref()
                .unwrap()
                .contains("Authentication expired")
        );
        let output = SubscriptionListOutput(tenants).into_cli_output();
        let json: serde_json::Value = serde_json::from_str(
            &output
                .render(Some(OutputFormat::Json), false)
                .unwrap()
                .unwrap(),
        )
        .unwrap();
        assert_eq!(json[1]["subscriptions"], serde_json::json!([]));
        assert!(json[1]["error"].is_null());
        assert!(
            json[2]["error"]
                .as_str()
                .unwrap()
                .contains("Authentication expired")
        );
        let text = output
            .render(Some(OutputFormat::Text), false)
            .unwrap()
            .unwrap();
        assert!(text.contains("Production"));
        assert!(text.contains("No subscriptions found."));
        assert!(text.contains("Unable to list subscriptions: Authentication expired"));
    }

    #[test]
    fn empty_tracking_is_actionable_and_terminal_control_characters_are_neutralized() {
        let output = SubscriptionListOutput(Vec::new())
            .render_text(false)
            .unwrap();
        assert!(output.contains("cloud_terrastodon az tenant discover"));
        assert!(output.contains("cloud_terrastodon az tenant add"));
        let mut output = example_output();
        output.0[0].tenant_name = Some("Contoso\n\x1b]8;;bad\x07".to_owned());
        let text = output.render_text(false).unwrap();
        assert!(!text.contains('\x1b'));
        assert!(!text.contains('\x07'));
        assert_eq!(text.lines().count(), 2);
    }
}
