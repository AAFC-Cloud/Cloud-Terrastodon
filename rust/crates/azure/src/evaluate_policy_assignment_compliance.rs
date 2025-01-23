use crate::prelude::fetch_all_policy_assignments;
use crate::prelude::ResourceGraphHelper;
use cloud_terrastodon_core_azure_types::prelude::DistinctByScope;
use cloud_terrastodon_core_azure_types::prelude::PolicyAssignment;
use cloud_terrastodon_core_azure_types::prelude::Scope;
use cloud_terrastodon_core_command::prelude::CacheBehaviour;
use cloud_terrastodon_core_tofu_types::prelude::Sanitizable;
use cloud_terrastodon_core_user_input::prelude::pick;
use cloud_terrastodon_core_user_input::prelude::Choice;
use cloud_terrastodon_core_user_input::prelude::FzfArgs;
use eyre::Result;
use indoc::formatdoc;
use itertools::Itertools;
use serde::Deserialize;
use std::path::PathBuf;
use std::time::Duration;
use tracing::info;

pub async fn evaluate_policy_assignment_compliance() -> Result<()> {
    info!("Fetching policy assignments");
    let policy_assignments = fetch_all_policy_assignments().await?;

    let Choice {
        value: policy_assignment,
        ..
    } = pick(FzfArgs {
        choices: policy_assignments
            .into_values()
            .flatten()
            .distinct_by_scope()
            .map(|ass| Choice::<PolicyAssignment> {
                key: format!("{} {:?}", ass.name, ass.display_name),
                value: ass,
            })
            .collect(),
        prompt: None,
        header: Some("Choose policy to evaluate".to_string()),
    })?;

    info!(
        "Querying policy compliance for {} ({:?})",
        policy_assignment.name, policy_assignment.display_name
    );

    let query = formatdoc! {
        r#"
policyResources 
| where type =~ 'Microsoft.PolicyInsights/PolicyStates'
| where properties.policyAssignmentId =~ "{}"
| where properties.complianceState =~ "noncompliant"
| extend policy_definition_reference_id = tostring(properties.policyDefinitionReferenceId)
| extend resource_type = tostring(properties.resourceType)
| summarize found = count() by policy_definition_reference_id, resource_type
| order by found desc
        "#,
        policy_assignment.id.expanded_form()
    };
    #[derive(Deserialize)]
    struct ReferenceIdRow {
        policy_definition_reference_id: String,
        resource_type: String,
        found: u32,
    }
    let reference_ids = ResourceGraphHelper::new(
        query,
        CacheBehaviour::Some {
            path: PathBuf::from(format!(
                "policy-compliance-for-{}",
                policy_assignment.name.sanitize()
            )),
            valid_for: Duration::from_mins(15),
        },
    )
    .collect_all::<ReferenceIdRow>()
    .await?;

    let Choice {
        value: chosen_reference_id,
        ..
    } = pick(FzfArgs {
        choices: reference_ids
            .into_iter()
            .map(|row| Choice {
                key: format!(
                    "{:64} - {:64} - {} non-compliant resources",
                    row.policy_definition_reference_id, row.resource_type, row.found
                ),
                value: row,
            })
            .collect_vec(),
        prompt: None,
        header: Some("Choose an inner policy to review".to_string()),
    })?;

    info!(
        "Fetching resource compliance for {}",
        chosen_reference_id.policy_definition_reference_id
    );

    let query = formatdoc! {
        r#"
policyResources 
| where type =~ 'Microsoft.PolicyInsights/PolicyStates'
| where properties.policyAssignmentId =~ "{}"
| where properties.complianceState =~ "noncompliant"
| where properties.policyDefinitionReferenceId =~ "{}"
| where properties.resourceType =~ "{}"
| join kind=leftouter (
    resourcecontainers
    | where type =~ "microsoft.resources/subscriptions"
    | project subscriptionId, sub_name=name
) on $left.subscriptionId == $right.subscriptionId
| project
    resource_group_name=resourceGroup,
    subscription_id=subscriptionId,
    subscription_name=sub_name,
    resource_id=properties.resourceId
        "#,
        policy_assignment.id.expanded_form(),
        chosen_reference_id.policy_definition_reference_id,
        chosen_reference_id.resource_type
    };
    #[derive(Deserialize)]
    struct ResourceRow {
        resource_group_name: String,
        subscription_id: String,
        subscription_name: String,
        resource_id: String,
    }
    let resource_ids = ResourceGraphHelper::new(
        query,
        CacheBehaviour::Some {
            path: PathBuf::from(format!(
                "policy-compliance-for-{}-{}",
                policy_assignment.name.sanitize(),
                chosen_reference_id
                    .policy_definition_reference_id
                    .sanitize()
            )),
            valid_for: Duration::from_mins(15),
        },
    )
    .collect_all::<ResourceRow>()
    .await?;

    let Choice {
        value: chosen_resource_id,
        ..
    } = pick(FzfArgs {
        choices: resource_ids
            .into_iter()
            .map(|row| Choice {
                key: format!(
                    "{:16} - {:64} - {}",
                    row.subscription_name,
                    row.resource_group_name,
                    row.resource_id
                        .rsplit_once("/")
                        .map(|x| x.1)
                        .unwrap_or(row.resource_id.as_str())
                ),
                value: row,
            })
            .collect_vec(),
        prompt: Some("Choose an inner policy to review> ".to_string()),
        header: Some(format!(
            "{} - {}",
            chosen_reference_id.policy_definition_reference_id, chosen_reference_id.resource_type,
        )),
    })?;

    info!(
        "You chose: {} - {}",
        chosen_resource_id.subscription_id, chosen_resource_id.resource_id
    );

    Ok(())
}
