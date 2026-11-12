use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::AzurePolicyAssignmentListRequest;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use eyre::bail;
use facet::Facet;
use std::sync::Arc;
use tracing::info;

/// Arguments for showing a single Azure policy assignment.
#[derive(Facet, Debug, Clone)]
pub struct AzurePolicyAssignmentShowArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,

    /// Policy assignment id, name, or display name.
    #[facet(figue::positional)]
    pub policy_assignment: String,
}

impl AzurePolicyAssignmentShowArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let authentication = self.authentication.resolve(executor).await?;
        info!(needle = %self.policy_assignment, tenant_id = %authentication.tenant_id, "Fetching Azure policy assignments");
        let policy_assignments = AzurePolicyAssignmentListRequest { authentication }
            .execute(executor)
            .await?;
        info!(
            count = policy_assignments.len(),
            "Fetched Azure policy assignments"
        );

        let needle = self.policy_assignment.trim();
        if let Some(policy_assignment) = policy_assignments
            .iter()
            .find(|policy_assignment| policy_assignment.id.expanded_form() == needle)
        {
            return Ok(CliOutput::facet(Arc::clone(policy_assignment)));
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
                        .as_deref()
                        .is_some_and(|display_name| display_name.eq_ignore_ascii_case(needle))
                })
                .collect::<Vec<_>>();
        }

        match name_matches.len() {
            0 => {
                bail!("No policy assignment found matching '{}'.", needle);
            }
            1 => Ok(CliOutput::facet(Arc::clone(name_matches.remove(0)))),
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
                );
            }
        }
    }
}
