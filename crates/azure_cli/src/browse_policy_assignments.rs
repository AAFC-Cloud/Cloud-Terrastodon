use crate::policy_compliance_row::PolicyComplianceRow;
use crate::policy_compliance_row::fetch_all_policy_compliance;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure::AzurePolicyAssignmentListRequest;
use cloud_terrastodon_azure::AzurePolicyDefinitionListRequest;
use cloud_terrastodon_azure::AzurePolicySetDefinitionListRequest;
use cloud_terrastodon_azure::PolicyAssignmentId;
use cloud_terrastodon_azure::PolicyDefinitionIdReference;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_user_input::PickResultExt;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Context;
use indexmap::IndexMap;
use itertools::Itertools;
use std::collections::HashMap;
use std::collections::HashSet;
use tracing::info;
use tracing::trace;
use tracing::warn;

/// This is a new function that merges “search assigned policies” with a compliance query.
pub async fn browse_policy_assignments(
    authentication: AzureBearerAuthenticationSelection,
    executor: &CloudTerrastodonExecutor,
) -> eyre::Result<CliOutput> {
    info!("Fetching a bunch of data...");
    let (policy_assignments, policy_definitions, policy_set_definitions, policy_compliance) =
        executor
            .join4(
                |child| async move {
                    AzurePolicyAssignmentListRequest { authentication }
                        .execute(&child)
                        .await
                },
                |child| async move {
                    AzurePolicyDefinitionListRequest { authentication }
                        .execute(&child)
                        .await
                },
                |child| async move {
                    AzurePolicySetDefinitionListRequest { authentication }
                        .execute(&child)
                        .await
                },
                |child| async move { fetch_all_policy_compliance(authentication, &child).await },
            )
            .await
            .context("failed to fetch policy data")?;

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
        HashMap<Option<String>, &PolicyComplianceRow>,
    > = policy_compliance
        .iter()
        .fold(HashMap::new(), |mut acc, elem| {
            acc.entry(&elem.policy_assignment_id)
                .or_default()
                .entry(
                    elem.policy_definition_reference_id
                        .as_ref()
                        .map(|id| id.to_ascii_lowercase()),
                )
                .or_insert(elem.as_ref());
            acc
        });

    info!("Found {} policy assignments", policy_assignments.len());
    info!("Found {} policy definitions", policy_definitions.len());
    info!(
        "Found {} policy set definitions",
        policy_set_definitions.len()
    );

    let mut choices = HashSet::new();
    for ass in policy_assignments {
        let mut row = IndexMap::<&str, String>::new();
        row.insert("ass id", ass.id.expanded_form().to_owned());
        if let Some(description) = ass
            .properties
            .description
            .as_deref()
            .filter(|v| !v.is_empty())
        {
            row.insert("ass desc", description.to_string());
        }
        row.insert(
            "ass dispaly name",
            ass.properties
                .display_name
                .as_deref()
                .unwrap_or(ass.name.as_str())
                .to_owned(),
        );
        row.insert("ass scope", ass.properties.scope.expanded_form());
        match &ass.properties.policy_definition_id {
            PolicyDefinitionIdReference::PolicyDefinitionId(policy_definition_id) => {
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
                        .as_deref()
                        .unwrap_or(policy_definition.name.as_str())
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
            PolicyDefinitionIdReference::PolicySetDefinitionId(policy_set_definition_id) => {
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
                            .as_deref()
                            .unwrap_or(policy_set_definition.name.as_str())
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
                            .as_deref()
                            .unwrap_or(policy_definition.name.as_str())
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
                        inner_definition
                            .policy_definition_reference_id
                            .to_ascii_lowercase(),
                    );
                    if let Some(compliance) = policy_compliance_map.get(&ass.id) {
                        if let Some(compliance) = compliance.get(&Some(
                            inner_definition
                                .policy_definition_reference_id
                                .to_ascii_lowercase(),
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
    let (chosen, maybe_error) = PickerTui::<_>::new()
        .pick_many(choices, executor.cancellation())
        .await
        .into_chosen_and_maybe_error()?;
    info!("You chose:\n{}", chosen.iter().join("\n=====\n"));
    Ok(CliOutput::facet(chosen).with_result(maybe_error))
}
