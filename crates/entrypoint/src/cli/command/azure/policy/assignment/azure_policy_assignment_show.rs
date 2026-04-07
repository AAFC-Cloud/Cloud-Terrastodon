use clap::Args;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_azure::fetch_all_policy_assignments;
use eyre::Result;
use eyre::bail;
use std::io::Write;
use tracing::info;

/// Arguments for showing a single Azure policy assignment.
#[derive(Args, Debug, Clone)]
pub struct AzurePolicyAssignmentShowArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[arg(long, default_value_t)]
    pub tenant: AzureTenantArgument<'static>,

    /// Policy assignment id, name, or display name.
    pub policy_assignment: String,
}

impl AzurePolicyAssignmentShowArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenant_id = self.tenant.resolve().await?;
        info!(needle = %self.policy_assignment, %tenant_id, "Fetching Azure policy assignments");
        let policy_assignments = fetch_all_policy_assignments(tenant_id).await?;
        info!(
            count = policy_assignments.len(),
            "Fetched Azure policy assignments"
        );

        let needle = self.policy_assignment.trim();
        if let Some(policy_assignment) = policy_assignments
            .iter()
            .find(|policy_assignment| policy_assignment.id.expanded_form() == needle)
        {
            let stdout = std::io::stdout();
            let mut handle = stdout.lock();
            serde_json::to_writer_pretty(&mut handle, policy_assignment)?;
            handle.write_all(b"\n")?;
            return Ok(());
        }

        let mut name_matches = policy_assignments
            .iter()
            .filter(|policy_assignment| {
                policy_assignment.name.as_str().eq_ignore_ascii_case(needle)
            })
            .collect::<Vec<_>>();
        if name_matches.is_empty() {
            name_matches = policy_assignments
                .iter()
                .filter(|policy_assignment| {
                    policy_assignment
                        .properties
                        .display_name
                        .as_str()
                        .eq_ignore_ascii_case(needle)
                })
                .collect::<Vec<_>>();
        }

        match name_matches.len() {
            0 => bail!("No policy assignment found matching '{}'.", needle),
            1 => {
                let stdout = std::io::stdout();
                let mut handle = stdout.lock();
                serde_json::to_writer_pretty(&mut handle, name_matches.remove(0))?;
                handle.write_all(b"\n")?;
                Ok(())
            }
            _ => {
                name_matches.sort_by_key(|policy_assignment| policy_assignment.id.expanded_form());
                let ids = name_matches
                    .iter()
                    .map(|policy_assignment| policy_assignment.id.expanded_form())
                    .collect::<Vec<_>>()
                    .join("\n  ");
                bail!(
                    "Multiple policy assignments matched '{}'. Use a full resource id.\n  {}",
                    needle,
                    ids
                )
            }
        }
    }
}
