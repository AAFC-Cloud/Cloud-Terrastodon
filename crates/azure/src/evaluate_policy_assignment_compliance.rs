use crate::prelude::ResourceGraphHelper;
use crate::prelude::fetch_all_policy_assignments;
use cloud_terrastodon_azure_types::prelude::DistinctByScope;
use cloud_terrastodon_azure_types::prelude::PolicyAssignment;
use cloud_terrastodon_azure_types::prelude::Scope;
use cloud_terrastodon_command::CacheBehaviour;
use cloud_terrastodon_hcl_types::prelude::Sanitizable;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use indoc::formatdoc;
use serde::Deserialize;
use std::path::PathBuf;
use std::time::Duration;
use tracing::info;

pub async fn evaluate_policy_assignment_compliance() -> Result<()> {
    info!("Fetching policy assignments");
    let policy_assignments = fetch_all_policy_assignments().await?;

    let policy_assignment: PolicyAssignment = PickerTui::new()
        .set_header("Choose policy to evaluate")
        .pick_one(policy_assignments.into_iter().distinct_by_scope().map(|ass| Choice::<PolicyAssignment> {
            key: format!("{} {:?}", ass.name, ass.properties.display_name),
            value: ass,
        }))?;

    info!(
        "Querying policy compliance for {} ({:?})",
        policy_assignment.name, policy_assignment.properties.display_name
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
            path: PathBuf::from_iter([
                "az",
                "resource_graph",
                format!(
                    "policy-compliance-for-{}",
                    policy_assignment.name.sanitize()
                )
                .as_str(),
            ]),
            valid_for: Duration::from_mins(15),
        },
    )
    .collect_all::<ReferenceIdRow>()
    .await?;

    let chosen_reference_id: ReferenceIdRow = PickerTui::new()
        .set_header("Choose an inner policy to review")
        .pick_one(reference_ids.into_iter().map(|row| Choice {
            key: format!(
                "{:64} - {:64} - {} non-compliant resources",
                row.policy_definition_reference_id, row.resource_type, row.found
            ),
            value: row,
        }))?;

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
    let resource_cache_key = format!(
        "policy-compliance-for-{}-{}",
        policy_assignment.name.sanitize(),
        chosen_reference_id
            .policy_definition_reference_id
            .sanitize()
    );
    let resource_ids = ResourceGraphHelper::new(
        query,
        CacheBehaviour::Some {
            path: PathBuf::from_iter([
                "az".to_string(),
                "resource_graph".to_string(),
                resource_cache_key,
            ]),
            valid_for: Duration::from_mins(15),
        },
    )
    .collect_all::<ResourceRow>()
    .await?;

    let chosen_resource_id: ResourceRow = PickerTui::new()
        .set_header(format!(
            "{} - {}",
            chosen_reference_id.policy_definition_reference_id, chosen_reference_id.resource_type,
        ))
        .pick_one(resource_ids.into_iter().map(|row| Choice {
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
        }))?;

    info!(
        "You chose: {} - {}",
        chosen_resource_id.subscription_id, chosen_resource_id.resource_id
    );

    Ok(())
}
