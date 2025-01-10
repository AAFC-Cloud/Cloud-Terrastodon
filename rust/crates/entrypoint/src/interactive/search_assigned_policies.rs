use anyhow::Context;
use cloud_terrastodon_core_azure::prelude::fetch_all_policy_assignments;
use cloud_terrastodon_core_azure::prelude::fetch_all_policy_definitions;
use cloud_terrastodon_core_azure::prelude::fetch_all_policy_set_definitions;
use cloud_terrastodon_core_azure::prelude::Scope;
use cloud_terrastodon_core_azure::prelude::ScopeImpl;
use cloud_terrastodon_core_azure::prelude::SomePolicyDefinitionId;
use cloud_terrastodon_core_user_input::prelude::pick_many;
use cloud_terrastodon_core_user_input::prelude::FzfArgs;
use indexmap::IndexMap;
use itertools::Itertools;
use std::collections::HashMap;
use std::collections::HashSet;
use tokio::try_join;
use tracing::info;
use tracing::warn;

pub async fn search_assigned_policies() -> anyhow::Result<()> {
    info!("Fetching policy assignments and definitions");
    let (policy_assignments, policy_definitions, policy_set_definitions) = match try_join!(
        fetch_all_policy_assignments(),
        fetch_all_policy_definitions(),
        fetch_all_policy_set_definitions()
    ) {
        Ok(x) => x,
        Err(e) => return Err(e).context("failed to fetch policy data"),
    };

    let policy_definition_map = policy_definitions
        .values()
        .flatten()
        .map(|v| (&v.id, v))
        .collect::<HashMap<_, _>>();
    let policy_set_definition_map = policy_set_definitions
        .values()
        .flatten()
        .map(|v| (&v.id, v))
        .collect::<HashMap<_, _>>();

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
            ass.display_name.as_ref().unwrap_or(&ass.name).to_owned()
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

                choices.insert(
                    row.iter()
                        .map(|(label, value)| format!("({label}) {value}"))
                        .join(" - "),
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

                    choices.insert(
                        row.iter()
                            .map(|(label, value)| format!("({label}) {value}"))
                            .join(" - "),
                    );
                }
            }
        }
    }
    let chosen = pick_many(FzfArgs {
        choices: choices.into_iter().collect_vec(),
        header: None,
        prompt: None,
    })?;
    info!(
        "You chose:\n{}",
        chosen
            .into_iter()
            .map(|x| x.split(" - ").join("\n"))
            .join("\n=====\n")
    );
    Ok(())
}
