use cloud_terrastodon_azure::prelude::PolicyAssignmentId;
use cloud_terrastodon_azure::prelude::ResourceGraphHelper;
use cloud_terrastodon_azure::prelude::Scope;
use cloud_terrastodon_azure::prelude::ScopeImpl;
use cloud_terrastodon_azure::prelude::SomePolicyDefinitionId;
use cloud_terrastodon_azure::prelude::fetch_all_policy_assignments;
use cloud_terrastodon_azure::prelude::fetch_all_policy_definitions;
use cloud_terrastodon_azure::prelude::fetch_all_policy_set_definitions;
use cloud_terrastodon_command::CacheBehaviour;
use cloud_terrastodon_user_input::FzfArgs;
use cloud_terrastodon_user_input::pick_many;
use eyre::Context;
use indexmap::IndexMap;
use indoc::indoc;
use itertools::Itertools;
use serde::Deserialize;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::PathBuf;
use std::time::Duration;
use tokio::try_join;
use tracing::info;
use tracing::trace;
use tracing::warn;

#[derive(Debug, Deserialize)]
struct PolicyComplianceRow {
    policy_assignment_id: PolicyAssignmentId,
    policy_definition_reference_id: Option<String>,
    compliant_count: u32,
    noncompliant_count: u32,
}

async fn fetch_all_policy_compliance() -> eyre::Result<Vec<PolicyComplianceRow>> {
    info!("Fetching all policy compliance information");
    let query = indoc! {r#"
policyResources
| where type =~ 'Microsoft.PolicyInsights/PolicyStates'
| where properties.complianceState !~ "Unknown"
| summarize 
    compliant_count = countif(tostring(properties.complianceState) == "Compliant"),
    noncompliant_count = countif(tostring(properties.complianceState) == "NonCompliant")
    by policy_assignment_id = tostring(properties.policyAssignmentId), 
       policy_definition_reference_id = tostring(properties.policyDefinitionReferenceId)
| project 
    policy_assignment_id,
    policy_definition_reference_id,
    compliant_count,
    noncompliant_count
    "#};

    let rtn = ResourceGraphHelper::new(
        query,
        CacheBehaviour::Some {
            path: PathBuf::from("policy-compliance"),
            valid_for: Duration::from_mins(15),
        },
    )
    .collect_all::<PolicyComplianceRow>()
    .await?;
    info!("Found {} policy compliance records", rtn.len());
    Ok(rtn)
}

/// This is a new function that merges “search assigned policies” with a compliance query.
pub async fn browse_policy_assignments() -> eyre::Result<()> {
    info!("Fetching a bunch of data...");
    let (policy_assignments, policy_definitions, mut policy_set_definitions, mut policy_compliance) =
        match try_join!(
            fetch_all_policy_assignments(),
            fetch_all_policy_definitions(),
            fetch_all_policy_set_definitions(),
            fetch_all_policy_compliance(),
        ) {
            Ok(x) => x,
            Err(e) => return Err(e).context("failed to fetch policy data"),
        };

    // make the policy definition reference IDs lowercase for equality sanity
    for ele in policy_set_definitions
        .iter_mut()
        .flat_map(|policy_set_definition| &mut policy_set_definition.policy_definitions)
        .flatten()
    {
        ele.policy_definition_reference_id.make_ascii_lowercase();
    }
    for ele in policy_compliance
        .iter_mut()
        .filter_map(|x| x.policy_definition_reference_id.as_mut())
    {
        ele.make_ascii_lowercase();
    }

    let policy_definition_map = policy_definitions
        .iter()
        .map(|v| (&v.id, v))
        .collect::<HashMap<_, _>>();
    let policy_set_definition_map = policy_set_definitions
        .iter()
        .map(|v| (&v.id, v))
        .collect::<HashMap<_, _>>();
    let policy_compliance_map: HashMap<
        &PolicyAssignmentId,
        HashMap<Option<&str>, &PolicyComplianceRow>,
    > = policy_compliance
        .iter()
        .fold(HashMap::new(), |mut acc, elem| {
            acc.entry(&elem.policy_assignment_id)
                .or_default()
                .entry(elem.policy_definition_reference_id.as_deref())
                .or_insert(elem);
            acc
        });

    info!("Found {} policy assignments", policy_assignments.len());
    info!("Found {} policy definitions", policy_definitions.len());
    info!(
        "Found {} policy set definitions",
        policy_set_definitions.len()
    );

    let mut choices = HashSet::new();
    for ass in policy_assignments.values().flatten() {
        let policy_definition_id = ass.policy_definition_id()?;
        let mut row = IndexMap::<&str, String>::new();
        row.insert("ass id", ass.id.expanded_form().to_owned());
        if let Some(desc) = &ass.description {
            row.insert("ass desc", desc.to_owned());
        }
        row.insert(
            "ass name",
            ass.display_name.as_ref().unwrap_or(&ass.name).to_owned(),
        );
        row.insert(
            "ass scope",
            ass.scope
                .parse::<ScopeImpl>()
                .map(|x| x.expanded_form().to_owned())
                .unwrap_or(ass.scope.to_owned()),
        );
        match policy_definition_id {
            SomePolicyDefinitionId::PolicyDefinitionId(policy_definition_id) => {
                let Some(policy_definition) = policy_definition_map.get(&policy_definition_id)
                else {
                    warn!(
                        "Couldn't find definition for policy {}",
                        policy_definition_id.expanded_form()
                    );
                    continue;
                };
                row.insert(
                    "pol name",
                    policy_definition
                        .display_name
                        .as_ref()
                        .unwrap_or(&policy_definition.name)
                        .to_owned(),
                );
                match &policy_definition.description {
                    Some(desc) if !desc.is_empty() => {
                        row.insert("pol desc", desc.to_owned());
                    }
                    _ => {}
                }
                if let Some(compliance) = policy_compliance_map.get(&ass.id) {
                    if let Some(compliance) = compliance.get(&None) {
                        row.insert(
                            "pol compliance",
                            format!(
                                "good={}\tbad={}",
                                compliance.compliant_count, compliance.noncompliant_count
                            ),
                        );
                    } else {
                        trace!(
                            "Failed to find compliance inside {}",
                            ass.id.expanded_form()
                        );
                    }
                } else {
                    trace!("Failed to find compliance for {}", ass.id.expanded_form());
                }

                choices.insert(
                    row.iter()
                        .map(|(label, value)| format!("({label}) {value}"))
                        .join("\n"),
                );
            }
            SomePolicyDefinitionId::PolicySetDefinitionId(policy_set_definition_id) => {
                let Some(policy_set_definition) =
                    policy_set_definition_map.get(&policy_set_definition_id)
                else {
                    warn!(
                        "Couldn't find policy set definition for policy {}",
                        policy_set_definition_id.expanded_form()
                    );
                    continue;
                };
                let Some(inner_definitions) = &policy_set_definition.policy_definitions else {
                    continue;
                };
                for inner_definition in inner_definitions {
                    let inner_definition_id = &inner_definition.policy_definition_id;
                    let Some(policy_definition) = policy_definition_map.get(&inner_definition_id)
                    else {
                        warn!(
                            "Couldn't find policy definition for policy {} inside policy set {}",
                            inner_definition_id.expanded_form(),
                            policy_set_definition_id.expanded_form()
                        );
                        continue;
                    };
                    row.insert(
                        "pol set name",
                        policy_set_definition
                            .display_name
                            .as_ref()
                            .unwrap_or(&policy_set_definition.name)
                            .to_owned(),
                    );

                    match &policy_set_definition.description {
                        Some(desc) if !desc.is_empty() => {
                            row.insert("pol set desc", desc.to_owned());
                        }
                        _ => {}
                    }
                    row.insert(
                        "pol name",
                        policy_definition
                            .display_name
                            .as_ref()
                            .unwrap_or(&policy_definition.name)
                            .to_owned(),
                    );
                    match &policy_definition.description {
                        Some(desc) if !desc.is_empty() => {
                            row.insert("pol desc", desc.to_owned());
                        }
                        _ => {}
                    }
                    row.insert(
                        "pol reference id",
                        inner_definition.policy_definition_reference_id.to_owned(),
                    );
                    if let Some(compliance) = policy_compliance_map.get(&ass.id) {
                        if let Some(compliance) = compliance.get(&Some(
                            inner_definition.policy_definition_reference_id.as_str(),
                        )) {
                            row.insert(
                                "pol compliance",
                                format!(
                                    "good={}\tbad={}",
                                    compliance.compliant_count, compliance.noncompliant_count
                                ),
                            );
                        } else {
                            trace!(
                                "Failed to find policy compliance for {} inside {}",
                                &inner_definition.policy_definition_reference_id,
                                ass.id.expanded_form()
                            );
                        }
                    } else {
                        trace!(
                            "Failed to find policy compliance for {}",
                            ass.id.expanded_form()
                        );
                    }
                    choices.insert(
                        row.iter()
                            .map(|(label, value)| format!("({label}) {value}"))
                            .join("\n"),
                    );
                }
            }
        }
    }
    let chosen = pick_many(FzfArgs {
        choices: choices.into_iter().collect_vec(),
        ..Default::default()
    })?;
    info!("You chose:\n{}", chosen.into_iter().join("\n=====\n"));
    Ok(())
}
