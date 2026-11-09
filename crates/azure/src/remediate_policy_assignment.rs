use crate::create_policy_remediation;
use crate::fetch_all_policy_assignments;
use crate::fetch_all_policy_set_definitions;
use cloud_terrastodon_auth::AzureTenantAuthContext;
use cloud_terrastodon_azure_types::DistinctByScope;
use cloud_terrastodon_azure_types::PolicyAssignment;
use cloud_terrastodon_azure_types::PolicyDefinitionIdReference;
use cloud_terrastodon_azure_types::PolicyRemediationId;
use cloud_terrastodon_azure_types::PolicyRemediationName;
use cloud_terrastodon_azure_types::PolicyRemediationScope;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use eyre::bail;
use eyre::eyre;
use itertools::Itertools;
use rand::Rng;
use std::sync::Arc;
use tracing::info;

/// Interactively select an assignment and submit its remediation tasks.
///
/// Tasks run asynchronously in Azure; see [policy remediation](https://learn.microsoft.com/en-us/azure/governance/policy/how-to/remediate-resources).
pub async fn remediate_policy_assignment(
    auth_context: &AzureTenantAuthContext,
    executor: &CloudTerrastodonExecutor,
) -> Result<()> {
    executor.cancellable_scope(|executor| async move {
        info!("Fetching policy assignments");
        let policy_assignments = fetch_all_policy_assignments(auth_context, &executor).await?;
        let choices = policy_assignments
            .into_iter()
            .distinct_by_scope()
            .map(|assignment| Choice::<Arc<PolicyAssignment>> {
                key: format!(
                    "{} {:?}",
                    assignment.name,
                    assignment.properties.display_name.as_deref()
                        .unwrap_or(assignment.name.as_str())
                ),
                value: assignment,
            });
        let policy_assignment: Arc<PolicyAssignment> = PickerTui::<_>::new()
            .set_header("Choose policy to remediate")
            .pick_one(choices, executor.cancellation())
            .await?;
        executor.cancellation().bail_if_cancelled()?;
        let remediation_scope = PolicyRemediationScope::try_from(&policy_assignment.properties.scope)?;

        let reference_ids = match &policy_assignment.properties.policy_definition_id {
            PolicyDefinitionIdReference::PolicySetDefinitionId(policy_set_definition_id) => {
                let Some(policy_set_definition) = fetch_all_policy_set_definitions(auth_context, &executor)
                    .await?.into_iter().find(|definition| definition.id == *policy_set_definition_id)
                else {
                    bail!("Could not find policy set definition with id {policy_set_definition_id:?}");
                };
                let choices = policy_set_definition.policy_definitions.as_ref().ok_or_else(|| eyre!(
                    "Expected {policy_set_definition_id:?} to have inner policy definitions"
                ))?.iter().map(|definition| Choice {
                    key: definition.policy_definition_reference_id.to_owned(),
                    value: definition,
                }).collect_vec();
                PickerTui::<_>::new()
                    .set_header("Pick the inner definitions to remediate")
                    .pick_many(choices, executor.cancellation())
                    .await?
                    .into_iter()
                    .map(|definition| definition.policy_definition_reference_id.parse().map(Some))
                    .collect::<Result<Vec<_>>>()?
            }
            PolicyDefinitionIdReference::PolicyDefinitionId(_) => vec![None],
        };

        info!("Launching remediation tasks");
        for reference_id in reference_ids {
            executor.cancellation().bail_if_cancelled()?;
            let id = PolicyRemediationId {
                scope: remediation_scope.clone(),
                name: PolicyRemediationName::new(format!("myRemediation{:x}", rand::rng().next_u32()))?,
            };
            info!(remediation = %id, "Submitting policy remediation task");
            create_policy_remediation(
                id,
                policy_assignment.id.clone(),
                reference_id,
                auth_context,
                &executor,
            ).await?;
        }
        Ok(())
    }).await
}
