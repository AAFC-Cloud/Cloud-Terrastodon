use clap::Args;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_azure::fetch_all_policy_set_definitions;
use eyre::Result;
use eyre::bail;
use std::io::Write;
use tracing::info;

/// Arguments for showing a single Azure policy set definition.
#[derive(Args, Debug, Clone)]
pub struct AzurePolicySetDefinitionShowArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[arg(long, default_value_t)]
    pub tenant: AzureTenantArgument<'static>,

    /// Policy set definition id, name, or display name.
    pub policy_set_definition: String,
}

impl AzurePolicySetDefinitionShowArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenant_id = self.tenant.resolve().await?;
        info!(needle = %self.policy_set_definition, %tenant_id, "Fetching Azure policy set definitions");
        let policy_set_definitions = fetch_all_policy_set_definitions(tenant_id).await?;
        info!(
            count = policy_set_definitions.len(),
            "Fetched Azure policy set definitions"
        );

        let needle = self.policy_set_definition.trim();
        if let Some(policy_set_definition) = policy_set_definitions
            .iter()
            .find(|policy_set_definition| policy_set_definition.id.expanded_form() == needle)
        {
            let stdout = std::io::stdout();
            let mut handle = stdout.lock();
            serde_json::to_writer_pretty(&mut handle, policy_set_definition)?;
            handle.write_all(b"\n")?;
            return Ok(());
        }

        let mut name_matches = policy_set_definitions
            .iter()
            .filter(|policy_set_definition| {
                policy_set_definition
                    .name
                    .as_str()
                    .eq_ignore_ascii_case(needle)
            })
            .collect::<Vec<_>>();
        if name_matches.is_empty() {
            name_matches = policy_set_definitions
                .iter()
                .filter(|policy_set_definition| {
                    policy_set_definition
                        .display_name
                        .as_deref()
                        .map(|display_name| display_name.eq_ignore_ascii_case(needle))
                        .unwrap_or(false)
                })
                .collect::<Vec<_>>();
        }

        match name_matches.len() {
            0 => bail!("No policy set definition found matching '{}'.", needle),
            1 => {
                let stdout = std::io::stdout();
                let mut handle = stdout.lock();
                serde_json::to_writer_pretty(&mut handle, name_matches.remove(0))?;
                handle.write_all(b"\n")?;
                Ok(())
            }
            _ => {
                name_matches
                    .sort_by_key(|policy_set_definition| policy_set_definition.id.expanded_form());
                let ids = name_matches
                    .iter()
                    .map(|policy_set_definition| policy_set_definition.id.expanded_form())
                    .collect::<Vec<_>>()
                    .join("\n  ");
                bail!(
                    "Multiple policy set definitions matched '{}'. Use a full resource id.\n  {}",
                    needle,
                    ids
                )
            }
        }
    }
}
