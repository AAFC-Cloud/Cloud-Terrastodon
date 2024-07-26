use anyhow::anyhow;
use anyhow::bail;
use anyhow::Result;
use azure_types::prelude::DistinctByScope;
use azure_types::prelude::ManagementGroupId;
use azure_types::prelude::PolicyAssignment;
use azure_types::prelude::PolicyDefinitionId;
use azure_types::prelude::PolicySetDefinitionId;
use azure_types::prelude::ResourceGroupId;
use azure_types::prelude::Scope;
use command::prelude::CommandBuilder;
use command::prelude::CommandKind;
use fzf::pick;
use fzf::pick_many;
use fzf::Choice;
use fzf::FzfArgs;
use itertools::Itertools;
use rand::RngCore;
use tracing::info;

use crate::prelude::fetch_all_policy_assignments;
use crate::prelude::fetch_all_policy_set_definitions;

pub async fn remediate_policy_assignment() -> Result<()> {
    info!("Fetching policy assignments");
    let policy_assignments = fetch_all_policy_assignments().await?;

    info!("Building choices of policies to remediate");
    let choices = policy_assignments
        .into_values()
        .flatten()
        .distinct_by_scope()
        .map(|ass| Choice::<PolicyAssignment> {
            key: format!("{} {:?}", ass.name, ass.display_name),
            value: ass,
        })
        .collect();

    info!("Prompting for user choice");
    let Choice {
        value: policy_assignment,
        ..
    } = pick(FzfArgs {
        choices,
        prompt: None,
        header: Some("Choose policy to remediate".to_string()),
    })?;

    info!("Finding policy definition for chosen");
    match (
        PolicySetDefinitionId::try_from_expanded(&policy_assignment.policy_definition_id),
        PolicyDefinitionId::try_from_expanded(&policy_assignment.policy_definition_id),
    ) {
        (Ok(policy_set_definition_id), Err(_)) => {
            info!("Remediating a policy set - must prompt for inner choice");
            let Some(policy_set_definition) = fetch_all_policy_set_definitions()
                .await?
                .into_values()
                .flatten()
                .find(|def| def.id == policy_set_definition_id)
            else {
                bail!("Could not find policy set definition with id {policy_set_definition_id:?}");
            };

            info!("Found policy set definition - prompting for inner definitions to remediate");
            let reference_ids = policy_set_definition
                .policy_definitions
                .ok_or(anyhow!(
                    "Expected {policy_set_definition_id:?} to have inner policy definitions"
                ))?
                .into_iter()
                .map(|x| Choice {
                    key: x.policy_definition_reference_id.to_owned(),
                    value: x,
                })
                .collect_vec();
            let chosen = pick_many(FzfArgs {
                choices: reference_ids,
                prompt: None,
                header: Some("Pick the inner definitions to remediate".to_string()),
            })?;

            info!("Launching remediation tasks");
            for choice in chosen {
                let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
                cmd.args(["policy", "remediation", "create"]);
                cmd.args([
                    "--name",
                    format!("myRemediation{:x}", rand::thread_rng().next_u32()).as_ref(),
                ]);
                cmd.args(["--policy-assignment", policy_assignment.id.expanded_form()]);
                cmd.args([
                    "--definition-reference-id",
                    choice.value.policy_definition_reference_id.as_ref(),
                ]);
                let scope = &policy_assignment.scope;
                if let Ok(management_group_id) = ManagementGroupId::try_from_expanded(scope) {
                    cmd.args(["--management-group", management_group_id.short_form()]);
                } else if let Ok(resource_group_id) = ResourceGroupId::try_from_expanded(scope) {
                    cmd.args(["--resource-group", resource_group_id.short_form()]);
                } else {
                    bail!("Could not identify kind of scope (management group, resource group) for scope {scope}");
                }
                cmd.should_announce(true);
                cmd.run_raw().await?;
            }
        }
        (Err(_), Ok(_policy_definition_id)) => {
            info!("Remediating a policy definition");
            let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
            cmd.args(["policy", "remediation", "create"]);
            cmd.args([
                "--name",
                format!("myRemediation{:x}", rand::thread_rng().next_u32()).as_ref(),
            ]);
            cmd.args(["--policy-assignment", policy_assignment.id.expanded_form()]);

            let scope = &policy_assignment.scope;
            if let Ok(management_group_id) = ManagementGroupId::try_from_expanded(scope) {
                cmd.args(["--management-group", management_group_id.short_form()]);
            } else if let Ok(resource_group_id) = ResourceGroupId::try_from_expanded(scope) {
                cmd.args(["--resource-group", resource_group_id.short_form()]);
            } else {
                bail!("Could not identify kind of scope (management group, resource group) for scope {scope}");
            }
            cmd.should_announce(true);
            cmd.run_raw().await?;
        }
        (Ok(policy_set_definition_id), Ok(policy_definition_id)) => unreachable!(
            "ID must not be of two kinds: {policy_set_definition_id:?} and {policy_definition_id:?}"
        ),
        (Err(e_set), Err(e_def)) => {
            bail!("Could not determine policy definition kind: {e_set:?} and {e_def:?}");
        }
    }

    Ok(())
}
