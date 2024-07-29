use anyhow::anyhow;
use anyhow::Result;
use azure::prelude::fetch_all_resource_groups;
use azure::prelude::fetch_all_subscriptions;
use fzf::pick_many;
use fzf::Choice;
use fzf::FzfArgs;
use std::collections::HashMap;
use tracing::info;

pub async fn browse_resource_groups() -> Result<()> {
    info!("Fetching subscriptions");
    let subscriptions = fetch_all_subscriptions()
        .await?
        .into_iter()
        .map(|sub| (sub.id.to_owned(), sub))
        .collect::<HashMap<_, _>>();
    info!("Fetching resource groups");
    let resource_groups = fetch_all_resource_groups().await?;

    info!("Building choices");
    let mut choices = Vec::new();
    for rg in resource_groups {
        let subscription_name = &subscriptions
            .get(&rg.subscription_id)
            .ok_or_else(|| {
                anyhow!(
                    "Failed to find name for subscription with id {}",
                    rg.subscription_id
                )
            })?
            .name;
        choices.push(Choice {
            key: format!(
                "{:90} - {:16} - {}",
                rg.name.to_owned(),
                subscription_name,
                rg.id
            ),
            value: (rg, subscription_name),
        });
    }
    // sort by subscription id
    choices.sort_by(|c1, c2| c1.1.cmp(c2.1));
    // sort by resource group name
    choices.sort_by(|c1, c2| c1.0.name.cmp(&c2.0.name));

    info!("Prompting user");
    let chosen = pick_many(FzfArgs {
        choices,
        prompt: None,
        header: Some("Browsing resource groups".to_string()),
    })?;

    info!("You chose:");
    for rg in chosen {
        info!("{} - {} - {}", rg.0.name.to_owned(), rg.1, rg.0.id);
    }

    Ok(())
}
