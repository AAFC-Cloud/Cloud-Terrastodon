use crate::prelude::fetch_all_policy_assignments;
use crate::prelude::QueryBuilder;
use anyhow::Result;
use azure_types::prelude::DistinctByScope;
use azure_types::prelude::PolicyAssignment;
use azure_types::prelude::Scope;
use command::prelude::CacheBehaviour;
use fzf::pick;
use fzf::Choice;
use fzf::FzfArgs;
use indoc::formatdoc;
use itertools::Itertools;
use serde::Deserialize;
use std::path::PathBuf;
use std::time::Duration;
use tracing::info;

pub async fn evaluate_policy_assignment_compliance() -> Result<()> {
    info!("Fetching policy assignments");
    let policy_assignments = fetch_all_policy_assignments().await?;

    info!("Building policy choice list");
    let choices = policy_assignments
        .into_values()
        .flatten()
        .distinct_by_scope()
        .map(|ass| Choice::<PolicyAssignment> {
            display: format!("{} {:?}", ass.name, ass.display_name),
            inner: ass,
        })
        .collect();

    info!("Prompting for user choice");
    let Choice {
        inner: policy_assignment,
        ..
    } = pick(FzfArgs {
        choices,
        prompt: None,
        header: Some("Choose policy to evaluate".to_string()),
    })?;

    info!("You chose: {:?}", policy_assignment.id);

    let query = formatdoc! {
        r#"
            policyResources 
            | where type =~ 'Microsoft.PolicyInsights/PolicyStates'
            | where properties.policyAssignmentId =~ "{}"
            | where properties.complianceState =~ "noncompliant"
            | extend policy_definition_reference_id = tostring(properties.policyDefinitionReferenceId)
            | summarize found = count() by policy_definition_reference_id
            | order by found desc
        "#,
        policy_assignment.id.expanded_form()
    };
    #[derive(Deserialize)]
    struct Row {
        policy_definition_reference_id: String,
        found: u32,	
    }
    let data = QueryBuilder::new(
        query.to_string(),
        CacheBehaviour::Some {
            path: PathBuf::from(format!(
                "--graph-query policy-compliance-for-{}",
                policy_assignment.name
            )),
            valid_for: Duration::from_mins(15),
        },
    )
    .collect_all::<Row>()
    .await?;

    let choice = pick(FzfArgs {
        choices: data
            .into_iter()
            .map(|row| Choice {
                display: format!(
                    "{} - {} non-compliant resources",
                    row.policy_definition_reference_id, row.found
                ),
                inner: row,
            })
            .collect_vec(),
        prompt: None,
        header: Some("Choose an inner policy to review".to_string()),
    })?;

    info!("You chose: {}", choice.inner.policy_definition_reference_id);

    Ok(())
}
