use clap::Args;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_azure::fetch_all_policy_definitions;
use eyre::Result;
use eyre::bail;
use std::io::Write;
use tracing::info;

/// Arguments for showing a single Azure policy definition.
#[derive(Args, Debug, Clone)]
pub struct AzurePolicyDefinitionShowArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[arg(long, default_value_t)]
    pub tenant: AzureTenantArgument<'static>,

    /// Policy definition id, name, or display name.
    pub policy_definition: String,
}

impl AzurePolicyDefinitionShowArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenant_id = self.tenant.resolve().await?;
        info!(needle = %self.policy_definition, %tenant_id, "Fetching Azure policy definitions");
        let policy_definitions = fetch_all_policy_definitions(tenant_id).await?;
        info!(
            count = policy_definitions.len(),
            "Fetched Azure policy definitions"
        );

        let needle = self.policy_definition.trim();
        if let Some(policy_definition) = policy_definitions
            .iter()
            .find(|policy_definition| policy_definition.id.expanded_form() == needle)
        {
            let stdout = std::io::stdout();
            let mut handle = stdout.lock();
            serde_json::to_writer_pretty(&mut handle, policy_definition)?;
            handle.write_all(b"\n")?;
            return Ok(());
        }

        let mut name_matches = policy_definitions
            .iter()
            .filter(|policy_definition| {
                policy_definition.name.as_str().eq_ignore_ascii_case(needle)
            })
            .collect::<Vec<_>>();
        if name_matches.is_empty() {
            name_matches = policy_definitions
                .iter()
                .filter(|policy_definition| {
                    policy_definition
                        .display_name
                        .as_deref()
                        .map(|display_name| display_name.eq_ignore_ascii_case(needle))
                        .unwrap_or(false)
                })
                .collect::<Vec<_>>();
        }

        match name_matches.len() {
            0 => bail!("No policy definition found matching '{}'.", needle),
            1 => {
                let stdout = std::io::stdout();
                let mut handle = stdout.lock();
                serde_json::to_writer_pretty(&mut handle, name_matches.remove(0))?;
                handle.write_all(b"\n")?;
                Ok(())
            }
            _ => {
                name_matches.sort_by_key(|policy_definition| policy_definition.id.expanded_form());
                let ids = name_matches
                    .iter()
                    .map(|policy_definition| policy_definition.id.expanded_form())
                    .collect::<Vec<_>>()
                    .join("\n  ");
                bail!(
                    "Multiple policy definitions matched '{}'. Use a full resource id.\n  {}",
                    needle,
                    ids
                )
            }
        }
    }
}
