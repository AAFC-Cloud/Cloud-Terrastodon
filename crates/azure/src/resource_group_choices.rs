use crate::prelude::fetch_all_resource_groups;
use crate::prelude::fetch_all_subscriptions;
use cloud_terrastodon_azure_types::prelude::ResourceGroup;
use cloud_terrastodon_azure_types::prelude::Subscription;
use cloud_terrastodon_user_input::Choice;
use eyre::bail;
use std::collections::HashMap;
use tracing::info;

/// Returns (Resource group, Subscription name)
pub async fn get_resource_group_choices() -> eyre::Result<Vec<Choice<(ResourceGroup, Subscription)>>>
{
    let subscriptions = fetch_all_subscriptions()
        .await?
        .into_iter()
        .map(|sub| (sub.id.to_owned(), sub))
        .collect::<HashMap<_, _>>();
    info!("Fetching resource groups");
    let resource_groups = fetch_all_resource_groups().await?;

    let mut choices = Vec::new();
    for rg in resource_groups {
        let Some(subscription) = subscriptions.get(&rg.subscription_id) else {
            bail!(
                "Failed to find name for subscription with id {}",
                rg.subscription_id
            );
        };
        let subscription_name = subscription.name.clone();
        choices.push(Choice {
            key: format!(
                "{:90} - {:16} - {}",
                rg.name.to_owned(),
                subscription_name,
                rg.id
            ),
            value: (rg, subscription.to_owned()),
        });
    }
    // sort by subscription id
    choices.sort_by(|c1, c2| c1.1.name.cmp(&c2.1.name));
    // sort by resource group name
    choices.sort_by(|c1, c2| c1.0.name.cmp(&c2.0.name));

    Ok(choices)
}
