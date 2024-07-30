use std::collections::HashMap;
use std::collections::HashSet;

use anyhow::bail;
use anyhow::Result;
use azure::prelude::fetch_all_resource_groups;
use azure::prelude::fetch_all_role_assignments_v2;
use azure::prelude::fetch_all_role_definitions;
use azure::prelude::fetch_all_security_groups;
use azure::prelude::fetch_all_subscriptions;
use azure::prelude::Scope;
use fzf::pick;
use fzf::pick_many;
use fzf::Choice;
use fzf::FzfArgs;
use itertools::Itertools;
use tokio::join;
use tracing::info;

use crate::noninteractive::prelude::clean;

pub async fn resource_group_import_wizard_menu() -> Result<()> {
    info!("Start from scratch or keep existing imports?");
    match pick(FzfArgs {
        choices: vec!["start from scratch", "keep existing imports"],
        prompt: None,
        header: None,
    })? {
        "start from scratch" => {
            info!("Starting from scratch");
            clean().await?
        }
        "keep existing imports" => {
            info!("Keeping existing imports");
        }
        _ => unreachable!(),
    }

    info!("Fetching subscriptions and resource groups");
    let (subscriptions, resource_groups) =
        join!(fetch_all_subscriptions(), fetch_all_resource_groups());
    let resource_groups = resource_groups?;
    let subscriptions = subscriptions?
        .into_iter()
        .map(|sub| (sub.id.to_owned(), sub))
        .collect::<HashMap<_, _>>();

    info!("Building pick list");
    let mut resource_group_choices = Vec::new();
    for rg in resource_groups {
        let Some(sub) = subscriptions.get(&rg.subscription_id) else {
            bail!(
                "Failed to find subscription {} for resource group {}",
                rg.subscription_id,
                rg.name
            );
        };
        let choice = Choice {
            key: format!("{:16} {}", sub.name, rg.name),
            value: (rg, sub),
        };
        resource_group_choices.push(choice);
    }

    info!("Picking resource groups");
    let resource_groups = pick_many(FzfArgs {
        choices: resource_group_choices,
        prompt: Some("Pick which to import".to_string()),
        header: None,
    })?;
    info!("You chose {} resource groups", resource_groups.len());

    info!("Fetching role assignments, role definitions, security groups");
    let (role_assignments, role_definitions, security_groups) = join!(
        fetch_all_role_assignments_v2(),
        fetch_all_role_definitions(),
        fetch_all_security_groups()
    );

    info!("Identifying relevant role assignments");
    // Convert to lower because I don't trust Azure IDs to match case everywhere
    let resource_group_ids = resource_groups
        .iter()
        .map(|rg| rg.value.0.id.expanded_form().to_lowercase())
        .collect::<HashSet<_>>();
    let role_assignments = role_assignments?
        .into_iter()
        .filter(|ra| resource_group_ids.contains(&ra.scope.to_lowercase()))
        .collect_vec();
    let role_definitions = role_definitions?;

    Ok(())
}
